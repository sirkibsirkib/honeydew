use crate::{
    game::{Player, NUM_PLAYERS, NUM_TELEPORTERS},
    prelude::*,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ControllerIdx {
    Zero,
    One,
    Two,
}

pub struct Net {
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
        preferred_controller: ControllerIdx,
    },
    CtsUpdate {
        client_time: WrapInt,
        pos: Pos,
        vel: Vel,
    },
    StCUpdate {
        server_time: WrapInt,
        players: [Player; NUM_PLAYERS as usize],
        teleporters: [Pos; NUM_TELEPORTERS as usize],
        room_seed: u16,
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
        Self { io_buf: Vec::with_capacity(1_024), sc: NetSc::Server { rng: Rng::new(None) } }
    }
}
