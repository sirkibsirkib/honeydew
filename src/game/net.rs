use {
    crate::{
        game::{
            config::{Config, IfServer},
            Player, PlayerColor, World, NUM_PLAYERS, NUM_TELEPORTERS,
        },
        prelude::*,
    },
    std::net::{Ipv4Addr, SocketAddrV4, UdpSocket},
};

/////////

pub struct Net {
    inner: NetInner,
    local_server: Option<NetServer>, // if None, I am the client
}

struct NetInner {
    udp: UdpSocket, // nonblocking. bound. connected IFF client.
    io_buf: [u8; 2_048],
}

pub struct NetServer {
    rng: Rng,
    addr_prey: Option<SocketAddrV4>,     // connected, bound
    addr_predator: Option<SocketAddrV4>, // connected, bound
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetMsg {
    CtsHello {
        preferred_color: PlayerColor, // TODO
    },
    CtsUpdate {
        pos: Pos,
        vel: Vel,
        client_time: WrapInt,
    },
    StCUpdate {
        players: [Player; NUM_PLAYERS as usize],
        teleporters: [Pos; NUM_TELEPORTERS as usize],
        server_time: WrapInt,
        room_seed: u16,
        you_control: PlayerColor,
    },
}

//////////////
impl NetInner {
    fn client_send(&mut self, msg: &NetMsg) {
        todo!()
    }
}
impl Net {
    pub fn server_rng(&mut self) -> Option<&mut Rng> {
        if let Some(NetServer { rng, .. }) = &mut self.local_server {
            Some(rng)
        } else {
            None
        }
    }
    pub fn new(config: &Config) -> Self {
        let mut inner = NetInner {
            udp: UdpSocket::bind(if config.server_mode {
                config.if_server.server_addr
            } else {
                SocketAddrV4::new(Ipv4Addr::from([0; 4]), 0)
            })
            .expect("Failed to bind to addr"),
            io_buf: [0; 2048], // TODO uninit
        };
        let local_server = if config.server_mode {
            //
            Some(NetServer { rng: Rng::new(None), addr_predator: None, addr_prey: None })
        } else {
            // I am a client!
            inner.udp.connect(config.if_client.server_addr).unwrap();
            let hello = NetMsg::CtsHello { preferred_color: config.if_client.preferred_color };
            loop {
                inner.client_send(&hello);
                // let msg =
            }
            // TODO await incoming msg
            // loop {
            //     let msg =
            // }
            None
        };
        udp.set_nonblocking(true).unwrap();
        Self { udp, io_buf: [0; 2048], local_server }
    }

    pub fn update(&mut self, world: &mut World, controlling: PlayerColor) {
        if let Some(s) = &mut self.local_server {
        } else {
            // I am a client
        }
    }
}
