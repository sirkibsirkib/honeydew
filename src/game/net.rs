use {
    crate::{
        game::{Player, PlayerColor, NUM_PLAYERS, NUM_TELEPORTERS},
        prelude::*,
    },
    mio::Poll,
};

pub struct Net {
    poll: Poll,
    io_buf: Vec<u8>,
    sc: NetSc,
}

pub enum NetSc {
    Server { rng: Rng },
    Client {},
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetMsg {
    CtsHello {
        // preferred_controller: ControllerIdx, // TODO
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
    pub fn new_server() -> Self {
        Self {
            io_buf: Vec::with_capacity(1_024),
            sc: NetSc::Server { rng: Rng::new(None) },
            poll: Poll::new().unwrap(),
        }
    }
}
