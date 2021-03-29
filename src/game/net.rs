use {
    crate::{
        game::{
            config::{Config, IfClient, IfServer},
            Entities, Player, PlayerArr, PlayerColor, Room, World, MOV_SPEED,
        },
        prelude::*,
    },
    bincode::Options,
    std::{
        borrow::Cow,
        net::{SocketAddr, SocketAddrV4, UdpSocket},
    },
};

/////////////////////////////////////////////////////////
pub struct Net {
    io: Io,
    my_timestamp: Timestamp,
    sc: Sc,
}

type Timestamp = WrapInt;
/////////////////////////////////////////////////////////
struct Io {
    udp: UdpSocket, // nonblocking. bound. connected IFF client.
    buf: Vec<u8>,   // invariant: EMPTY if no message is prepared
}

enum Sc {
    Server(Server),
    Client { last_server_timestamp: Timestamp },
}

struct Server {
    rng: Rng,
    clients: PlayerArr<Option<ServerClient>>,
    room_seed: u64,
}
struct ServerClient {
    addr: SocketAddr,
    last_client_timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimelyGameData<'a> {
    entities: Cow<'a, Entities>,
    timestamp: Timestamp,
}

#[derive(Debug, Serialize, Deserialize)]
enum Msg<'a> {
    CtsHello { preferred_color: PlayerColor },
    StcHello { timely: TimelyGameData<'a>, your_color: PlayerColor, room_seed: u64 },
    CtsUpdate { player: Player, timestamp: WrapInt },
    StcUpdate { timely: TimelyGameData<'a> },
}
const SERVER_MOVE_THRESH: DimMap<u16> = {
    let mut res = MOV_SPEED;
    res.arr[0] *= 30;
    res.arr[1] *= 30;
    res
};

//////////////////////////////////////////////////////////////////////
fn bincode_config() -> impl bincode::config::Options {
    bincode::DefaultOptions::new().with_limit(1024).with_varint_encoding()
}
impl Io {
    const BUF_CAP: usize = 2048;
    pub fn nonblocking(self) -> Self {
        self.udp.set_nonblocking(true).unwrap();
        self
    }
    pub fn connected(self, server_addr: SocketAddr) -> Self {
        self.udp.connect(server_addr).expect("UDP Connect err");
        self
    }
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            udp: UdpSocket::bind(server_addr).expect("Failed to bind to addr"),
            buf: Vec::with_capacity(Self::BUF_CAP),
        }
    }
    pub fn with_staged_msg(&mut self, msg: &Msg, func: impl FnOnce(&mut [u8], &mut UdpSocket)) {
        bincode_config().serialize_into(&mut self.buf, msg).unwrap();
        func(self.buf.as_mut_slice(), &mut self.udp);
        self.buf.clear();
    }
    pub fn with_temp_cap_buf<R>(&mut self, func: impl FnOnce(&mut [u8], &mut UdpSocket) -> R) -> R {
        unsafe {
            // SAFE! u8 vector contents are P.O.D. with no invalid repr
            self.buf.set_len(Self::BUF_CAP);
        }
        let res = func(&mut self.buf, &mut self.udp);
        self.buf.clear();
        res
    }
    pub fn recv(&mut self) -> Option<Msg> {
        self.with_temp_cap_buf(|temp_buf, udp| match udp.recv(temp_buf) {
            Ok(0) | Err(_) => None,
            Ok(n) => bincode_config().deserialize(&temp_buf[..n]).ok(),
        })
    }
    pub fn recv_from(&mut self) -> Option<(Msg, SocketAddr)> {
        self.with_temp_cap_buf(|temp_buf, udp| match udp.recv_from(temp_buf) {
            Ok((0, _)) | Err(_) => None,
            Ok((n, addr)) => {
                bincode_config().deserialize(&temp_buf[..n]).ok().map(move |msg| (msg, addr))
            }
        })
    }
}
impl Net {
    pub fn server_rng(&mut self) -> Option<&mut Rng> {
        if let Sc::Server(server) = &mut self.sc {
            Some(&mut server.rng)
        } else {
            None
        }
    }

    pub fn new_server(config: &IfServer) -> (Self, World, PlayerColor) {
        let room_seed = config.room_seed.unwrap_or_else(Rng::random_seed);
        let (room, mut rng) = Room::new_seeded(room_seed);
        let entities = Entities::random(&mut rng);
        let world = World { room, entities };
        let net = Self {
            io: Io::new(config.server_addr.into()).nonblocking(),
            sc: Sc::Server(Server { room_seed, rng, clients: Default::default() }),
            my_timestamp: Timestamp::default(),
        };
        (net, world, config.player_color)
    }

    pub fn new_client(config: &IfClient) -> (Self, World, PlayerColor) {
        let mut io = Io::new(SocketAddrV4::new(std::net::Ipv4Addr::UNSPECIFIED, 0).into())
            .connected(config.server_addr.into());
        let hello = Msg::CtsHello { preferred_color: config.preferred_color };
        loop {
            io.with_staged_msg(&hello, |bytes, udp| {
                udp.send(bytes).unwrap();
            });
            if let Some(Msg::StcHello { timely, your_color, room_seed }) = io.recv() {
                let TimelyGameData { entities, timestamp } = timely;
                let (room, _rng) = Room::new_seeded(room_seed);
                let world = World { room, entities: entities.into_owned() };
                let net = Self {
                    io: io.nonblocking(),
                    sc: Sc::Client { last_server_timestamp: timestamp },
                    my_timestamp: Timestamp::default(),
                };
                return (net, world, your_color);
            }
        }
    }

    pub fn new(config: &Config) -> (Self, World, PlayerColor) {
        if config.server_mode {
            Self::new_server(&config.if_server)
        } else {
            Self::new_client(&config.if_client)
        }
    }

    pub fn update(&mut self, my_color: PlayerColor, entities: &mut Entities) {
        match &mut self.sc {
            Sc::Client { last_server_timestamp } => {
                // I am the client!
                // handle all incoming server update messages in the correct order
                while let Some(Msg::StcUpdate { timely }) = self.io.recv() {
                    if *last_server_timestamp < timely.timestamp {
                        // new info!
                        *last_server_timestamp = timely.timestamp;
                        // overwrite all entity data except my own
                        let my_old = entities.players[my_color].clone();
                        *entities = timely.entities.into_owned();
                        let my_new = &mut entities.players[my_color];
                        // ... but not my velocity (mine is always accurate)
                        my_new.vel = my_old.vel;
                        // client ignores updates representing SMALL STEPS
                        if my_old.pos.abs_differences(my_new.pos) < SERVER_MOVE_THRESH {
                            // the difference was a small step. RESTORE what I had before
                            my_new.pos = my_old.pos;
                        }
                    }
                }
                // update the server!
                let update_msg = Msg::CtsUpdate {
                    timestamp: self.my_timestamp,
                    player: entities.players[my_color].clone(),
                };
                self.io.with_staged_msg(&update_msg, |bytes, udp| {
                    udp.send(bytes).unwrap();
                });
            }
            Sc::Server(server) => {
                // I am the server!
                let peer_colors = std::array::IntoIter::new(my_color.peers());
                while let Some((msg, sender_addr)) = self.io.recv_from() {
                    match msg {
                        Msg::CtsHello { preferred_color } => {
                            // what color is the sender's player?
                            let timestamp = self.my_timestamp;
                            let your_color = PlayerColor::iter_domain()
                                // try 1: the color of a client with the sender's addr
                                .find(|&color| {
                                    server.clients[color]
                                        .as_ref()
                                        .map(|c| c.addr == sender_addr)
                                        .unwrap_or(false)
                                })
                                // try 2: color of a newly-filled client slot
                                .or_else(|| {
                                    let [b, c] = preferred_color.peers();
                                    let choices = [preferred_color, b, c];
                                    ArrIter::new(choices)
                                        .find(|&color| {
                                            color != my_color && server.clients[color].is_none()
                                        })
                                        .map(|color| {
                                            server.clients[color] = Some(ServerClient {
                                                addr: sender_addr,
                                                last_client_timestamp: timestamp,
                                            });
                                            color
                                        })
                                });
                            if let Some(your_color) = your_color {
                                // yes you've got a color! Reply with info
                                let hello = Msg::StcHello {
                                    your_color,
                                    room_seed: server.room_seed,
                                    timely: TimelyGameData {
                                        entities: Cow::Borrowed(entities),
                                        timestamp: self.my_timestamp,
                                    },
                                };
                                self.io.with_staged_msg(&hello, |bytes, udp| {
                                    udp.send_to(bytes, sender_addr).unwrap();
                                });
                            } else {
                                // sorry, cannot support a new player/color
                            }
                        }
                        Msg::CtsUpdate { player, timestamp } => {
                            'find_player: for color in peer_colors.clone() {
                                if let Some(client) = &mut server.clients[color] {
                                    if client.addr == sender_addr {
                                        // found them!
                                        if client.last_client_timestamp < timestamp {
                                            // update player data with newer info!
                                            let curr_player = &mut entities.players[color];
                                            // server accepts SMALL STEPS, ignores LARGE JUMPS
                                            if curr_player.pos.abs_differences(player.pos)
                                                < SERVER_MOVE_THRESH
                                            {
                                                // the step was SMALL (keep it!)
                                                curr_player.pos = player.pos;
                                            }
                                            curr_player.vel = player.vel;
                                            client.last_client_timestamp = timestamp;
                                        }
                                        break 'find_player;
                                    }
                                }
                            }
                        }
                        Msg::StcHello { .. } | Msg::StcUpdate { .. } => { /* ignore */ }
                    }
                }
                // update all clients!
                let update_msg = Msg::StcUpdate {
                    timely: TimelyGameData {
                        entities: Cow::Borrowed(entities),
                        timestamp: self.my_timestamp,
                    },
                };
                self.io.with_staged_msg(&update_msg, |bytes, udp| {
                    for color in peer_colors {
                        if let Some(client) = &mut server.clients[color] {
                            udp.send_to(bytes, client.addr).unwrap();
                        }
                    }
                });
            }
        }
        self.my_timestamp += 1u16;
    }
}

impl Pos {
    fn abs_differences(self, other: Self) -> DimMap<u16> {
        DimMap::new_xy_with(|dim| (self[dim] - other[dim]).distance_from_zero())
    }
}
