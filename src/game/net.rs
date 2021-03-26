use {
    crate::{
        game::{config::Config, Player, PlayerColor, World, NUM_PLAYERS, NUM_TELEPORTERS},
        prelude::*,
    },
    std::net::{SocketAddrV4, UdpSocket},
};

/////////

pub struct Net {
    udp: UdpSocket, // nonblocking. bound. connected IFF client.
    io_buf: [u8; 2_048],
    local_server: Option<NetServer>, // if None, I am the client
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
impl Net {
    pub fn server_rng(&mut self) -> Option<&mut Rng> {
        if let Some(NetServer { rng, .. }) = &mut self.local_server {
            Some(rng)
        } else {
            None
        }
    }
    fn new_server(config: &Config) -> NetServer {
        NetServer { rng: Rng::new(None), addr_predator: None, addr_prey: None }
    }
    pub fn new(config: &Config) -> Self {
        let udp =
            UdpSocket::bind(config.server_addr_if_server).expect("Failed to bind to server addr");
        udp.set_nonblocking(true).unwrap();
        Self { udp, io_buf: [0; 2048], local_server: Some(Self::new_server(config)) }
    }

    pub fn update(&mut self, world: &mut World, controlling: PlayerColor) {
        //     self.poll.poll(&mut self.events, TIMEOUT).unwrap();
        // match &mut self.sc {
        //     NetSc::Server {  } => {
        //             for event in &self.events {
        //                 match event.token() {
        //                     TOKEN_S_LISTENER => {
        //                         match Self::try_recv_msg(&mut self.io_buf, listener_udp) {
        //                             None => {}
        //                             Some(NetMsg::CtsHello { preferred_color }) => {}
        //                             // Some(NetMsg::Cts)
        //                             _ => todo!(),
        //                         }
        //                     }
        //                     TOKEN_S_PREDATOR => todo!(),
        //                     TOKEN_S_PREY => todo!(),
        //                     _ => unreachable!(),
        //                 }
        //             }
        //             // 1 accept new incoming connections
        //             todo!() //
        //         }
        //     NetSc::Client {  } => {
        //         todo!()
        //     }
        // }
    }
}
