use core::time::Duration;
use {
    crate::{
        game::{config::Config, Player, PlayerColor, World, NUM_PLAYERS, NUM_TELEPORTERS},
        prelude::*,
    },
    mio::{net::UdpSocket, Events, Interest, Poll, Token},
};

const TIMEOUT: Option<Duration> = Some(Duration::from_secs(0));
const TOKEN_S_LISTENER: Token = Token(0);
const TOKEN_S_PREDATOR: Token = Token(1);
const TOKEN_S_PREY: Token = Token(2);

const TOKEN_C_SERVER: Token = Token(0);

/////////

pub struct Net {
    events: Events,
    poll: Poll,
    io_buf: Vec<u8>,
    sc: NetSc,
}

pub enum NetSc {
    Server {
        listener_udp: UdpSocket, // bound
        rng: Rng,
        client_prey: Option<UdpSocket>,     // connected, bound
        client_predator: Option<UdpSocket>, // connected, bound
    },
    Client {
        server_udp: UdpSocket, // bound, connected
    },
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
        if let NetSc::Server { rng, .. } = &mut self.sc {
            Some(rng)
        } else {
            None
        }
    }
    pub fn new_server(config: &Config) -> Self {
        let mut listener_udp = UdpSocket::bind(config.server_ip_when_server.into())
            .expect("Failed to bind to the given server ip!");
        let poll = Poll::new().unwrap();
        poll.registry()
            .register(&mut listener_udp, TOKEN_S_LISTENER, Interest::READABLE | Interest::WRITABLE)
            .expect("Failed to register server's listener socket!");
        Self {
            events: Events::with_capacity(16),
            io_buf: Vec::with_capacity(1_024),
            sc: NetSc::Server {
                rng: Rng::new(None),
                client_predator: None,
                client_prey: None,
                listener_udp,
            },
            poll,
        }
    }

    fn try_recv_msg(buf: &mut Vec<u8>, sock: &mut UdpSocket) -> Option<NetMsg> {
        buf.clear();
        match sock.recv(buf) {
            Err(_) | Ok(0) => None,
            Ok(_) => bincode::deserialize(buf).ok(),
        }
    }

    pub fn update(&mut self, world: &mut World, controlling: PlayerColor) {
        self.poll.poll(&mut self.events, TIMEOUT).unwrap();
        match &mut self.sc {
            NetSc::Server { listener_udp, client_prey, client_predator, .. } => {
                for event in &self.events {
                    match event.token() {
                        TOKEN_S_LISTENER => {
                            match Self::try_recv_msg(&mut self.io_buf, listener_udp) {
                                None => {}
                                Some(NetMsg::CtsHello { preferred_color }) => {}
                                // Some(NetMsg::Cts)
                                _ => todo!(),
                            }
                        }
                        TOKEN_S_PREDATOR => todo!(),
                        TOKEN_S_PREY => todo!(),
                        _ => unreachable!(),
                    }
                }
                // 1 accept new incoming connections
                todo!() //
            }
            NetSc::Client { server_udp } => {
                todo!()
            }
        }
    }
}
