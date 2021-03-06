use {
    crate::{
        game::{
            config::{IfClient, IfServer},
            Entities, Player, PlayerArr, PlayerColor, Room, World, MOVE_SIZE,
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

type Timestamp = WrapInt;
/////////////////////////////////////////////////////////
struct Io {
    udp: UdpSocket, // nonblocking. bound. connected IFF client.
    buf: Vec<u8>,   // invariant: EMPTY if no message is prepared
}

pub struct Client {
    io: Io, // nonblocking && bound && connected
    server_ts: Timestamp,
    client_ts: Timestamp,
}

pub struct Server {
    io: Io, // nonblocking && bound
    clients: PlayerArr<Option<ServerClient>>,
    room_seed: u64,
    server_ts: Timestamp,
}
struct ServerClient {
    addr: SocketAddr,
    client_ts: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
struct StcTimelyData<'a> {
    entities: Cow<'a, Entities>,
    server_ts: Timestamp,
}

#[derive(Debug, Serialize, Deserialize)]
enum Msg<'a> {
    CtsHello {
        client_ts: Timestamp,
        preferred_color: PlayerColor,
    },
    StcHello {
        server_ts: Timestamp,
        room_seed: u64,
        server_entities: Cow<'a, Entities>,
        your_color: PlayerColor,
    },
    CtsUpdate {
        player: Player,
        client_ts: Timestamp,
    },
    StcUpdate {
        server_ts: Timestamp,
        server_entities: Cow<'a, Entities>,
    },
}

const ACCEPTED_CLIENT_MOVE: Size = MOVE_SIZE.scalar_mul(20);

//////////////////////////////////////////////////////////////////////
fn bincode_config() -> impl bincode::config::Options {
    bincode::DefaultOptions::new().with_limit(1024).with_varint_encoding()
}
impl Io {
    const BUF_CAP: usize = 2048;
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            udp: UdpSocket::bind(server_addr).expect("Failed to bind to addr"),
            buf: Vec::with_capacity(Self::BUF_CAP),
        }
    }
    pub fn nonblocking(self) -> Self {
        self.udp.set_nonblocking(true).unwrap();
        self
    }
    pub fn connected(self, server_addr: SocketAddr) -> Self {
        self.udp.connect(server_addr).expect("UDP Connect err");
        self
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
}
impl Client {
    fn recv(io: &mut Io) -> Option<Msg> {
        io.with_temp_cap_buf(|temp_buf, udp| match udp.recv(temp_buf) {
            Ok(0) | Err(_) => None,
            Ok(n) => bincode_config().deserialize(&temp_buf[..n]).ok(),
        })
    }
    pub fn new(config: &IfClient) -> (Self, World, PlayerColor) {
        let mut io = Io::new(SocketAddrV4::new(std::net::Ipv4Addr::UNSPECIFIED, 0).into())
            .connected(config.server_addr.into());
        let client_ts = Timestamp::default();
        let preferred_color = config.preferred_color;
        let hello = Msg::CtsHello { preferred_color, client_ts };
        loop {
            io.with_staged_msg(&hello, |bytes, udp| {
                udp.send(bytes).unwrap();
            });
            if let Some(Msg::StcHello { server_entities, server_ts, your_color, room_seed }) =
                Self::recv(&mut io)
            {
                let (room, _rng) = Room::new_seeded(room_seed);
                let world = World { room, entities: server_entities.into_owned() };
                let me = Self { io: io.nonblocking(), server_ts, client_ts };
                return (me, world, your_color);
            }
        }
    }
    pub fn update(&mut self, my_color: PlayerColor, entities: &mut Entities) {
        // handle all incoming server update messages in the correct order
        while let Some(Msg::StcUpdate { server_ts, server_entities }) = Self::recv(&mut self.io) {
            if self.server_ts < server_ts {
                // new info!
                self.server_ts = server_ts;
                // overwrite all entity data except my own
                let my_old = entities.players[my_color].clone();
                *entities = server_entities.into_owned();
                let my_new = &mut entities.players[my_color];
                // ... but not my velocity (mine is always accurate)
                my_new.vel = my_old.vel;
                // client ignores updates representing SMALL STEPS
                let diff = my_old.pos - my_new.pos;
                if diff.distances_from_zero() < ACCEPTED_CLIENT_MOVE {
                    // the difference was a small step. RESTORE what I had before
                    my_new.pos = my_old.pos;
                }
            }
        }
        // update the server!
        let update_msg = Msg::CtsUpdate {
            client_ts: self.client_ts,
            player: entities.players[my_color].clone(),
        };
        self.io.with_staged_msg(&update_msg, |bytes, udp| {
            udp.send(bytes).unwrap();
        });
        self.client_ts += 1u16;
    }
}
impl Server {
    pub fn new(config: &IfServer) -> (Self, World, PlayerColor) {
        let room_seed = config.room_seed.unwrap_or_else(Rng::random_seed);
        let (room, mut rng) = Room::new_seeded(room_seed);
        let entities = Entities::random(&mut rng); // doesn't matter if local seed
        let world = World { room, entities };
        let me = Self {
            io: Io::new(config.server_addr.into()).nonblocking(),
            room_seed,
            clients: Default::default(),
            server_ts: Timestamp::default(),
        };
        (me, world, config.player_color)
    }
    fn recv_from(&mut self) -> Option<(Msg, SocketAddr)> {
        self.io.with_temp_cap_buf(|temp_buf, udp| match udp.recv_from(temp_buf) {
            Ok((0, _)) | Err(_) => None,
            Ok((n, addr)) => {
                bincode_config().deserialize(&temp_buf[..n]).ok().map(move |msg| (msg, addr))
            }
        })
    }
    pub fn update(
        &mut self,
        my_color: PlayerColor,
        entities: &mut Entities,
        mut new_client_callback: impl FnMut(PlayerColor, &mut Entities),
    ) {
        // I am the server!
        let peer_colors = std::array::IntoIter::new(my_color.predator_prey());
        while let Some((msg, sender_addr)) = self.recv_from() {
            match msg {
                Msg::CtsHello { preferred_color, client_ts } => {
                    // what color is the sender's player?
                    let server_ts = self.server_ts;
                    let your_color = PlayerColor::iter_domain()
                        // try 1: the color of a client with the sender's addr
                        .find(|&color| {
                            self.clients[color]
                                .as_ref()
                                .map(|c| c.addr == sender_addr)
                                .unwrap_or(false)
                        })
                        // try 2: color of a newly-filled client slot
                        .or_else(|| {
                            let [b, c] = preferred_color.predator_prey();
                            let choices = [preferred_color, b, c];
                            ArrIter::new(choices)
                                .find(|&color| color != my_color && self.clients[color].is_none())
                                .map(|color| {
                                    self.clients[color] =
                                        Some(ServerClient { addr: sender_addr, client_ts });
                                    new_client_callback(color, entities);
                                    color
                                })
                        });
                    if let Some(your_color) = your_color {
                        // yes you've got a color! Reply with info
                        let hello = Msg::StcHello {
                            your_color,
                            room_seed: self.room_seed,
                            server_entities: Cow::Borrowed(entities),
                            server_ts: self.server_ts,
                        };
                        self.io.with_staged_msg(&hello, |bytes, udp| {
                            udp.send_to(bytes, sender_addr).unwrap();
                        });
                    } else {
                        // sorry, cannot support a new player/color
                    }
                }
                Msg::CtsUpdate { player, client_ts } => {
                    'find_player: for color in peer_colors.clone() {
                        if let Some(client) = &mut self.clients[color] {
                            if client.addr == sender_addr {
                                // found them!
                                if client.client_ts < client_ts {
                                    // update player data with newer info!
                                    let curr_player = &mut entities.players[color];
                                    // server accepts SMALL STEPS, ignores LARGE JUMPS
                                    let diff = curr_player.pos - player.pos;
                                    if diff.distances_from_zero() < ACCEPTED_CLIENT_MOVE {
                                        // the step was SMALL (keep it!)
                                        curr_player.pos = player.pos;
                                    }
                                    curr_player.vel = player.vel;
                                    client.client_ts = client_ts;
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
        let update_msg =
            Msg::StcUpdate { server_entities: Cow::Borrowed(entities), server_ts: self.server_ts };
        let Self { clients, io, .. } = self;
        io.with_staged_msg(&update_msg, |bytes, udp| {
            for color in peer_colors {
                if let Some(client) = &mut clients[color] {
                    udp.send_to(bytes, client.addr).unwrap();
                }
            }
        });
        self.server_ts += 1u16;
    }
}
