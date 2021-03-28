mod axes;
mod bit_set;
mod game;
mod prelude;
mod rng;
mod wrap_int;

use crate::game::config::IfClient;
use crate::game::config::IfServer;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use {
    crate::{
        game::{config::Config, rendering::render_config, GameState, PlayerColor},
        prelude::*,
    },
    gfx_2020::{gfx_hal::Backend, *},
};
/////////////////////////////////

pub(crate) fn game_state_init_fn<B: Backend>(
    renderer: &mut Renderer<B>,
) -> ProceedWith<&'static mut GameState> {
    let server_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let config = Config {
        server_mode: true,
        if_client: IfClient {
            preferred_color: PlayerColor::Black,
            connect_timeout: Duration::from_secs(3),
            server_addr,
        },
        if_server: IfServer { specified_seed: None, player_color: PlayerColor::Black, server_addr },
    };
    // TODO
    let seed = 1;
    Ok(Box::leak(Box::new(GameState::new_seeded(renderer, seed, &config))))
}

fn main() {
    gfx_2020::main_loop::<gfx_backend_vulkan::Backend, _, _>(&render_config(), game_state_init_fn);
}
