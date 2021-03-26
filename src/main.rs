mod axes;
mod bit_set;
mod game;
mod prelude;
mod rng;
mod wrap_int;

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
    let config = Config {
        preferred_color: PlayerColor::Black,
        server_addr_if_client: "0.0.0.0:0".parse().unwrap(),
        server_addr_if_server: "0.0.0.0:0".parse().unwrap(),
        server_mode: true,
    };
    let maybe_seed = Some(1);
    Ok(Box::leak(Box::new(GameState::new(renderer, maybe_seed, &config))))
}

fn main() {
    gfx_2020::main_loop::<gfx_backend_vulkan::Backend, _, _>(&render_config(), game_state_init_fn);
}
