////////////////////////////
mod basic;
mod bit_set;
mod game;
mod prelude;
mod rng;
mod wrap_fields;

use {
    crate::{
        game::{rendering::render_config, GameState, Net, World},
        prelude::*,
        rng::Rng,
    },
    gfx_2020::{gfx_hal::Backend, *},
};
/////////////////////////////////

pub(crate) fn game_state_init_fn<B: Backend>(
    renderer: &mut Renderer<B>,
) -> ProceedWith<&'static mut GameState> {
    let texture = gfx_2020::load_texture_from_path("./src/data/faces.png").unwrap();
    let tex_id = renderer.load_texture(&texture);
    let maybe_seed = Some(1);
    let state = GameState {
        net: Net::Server { rng: Rng::new(None) },
        world: World::random(maybe_seed),
        pressing_state: Default::default(),
        tex_id,
        draw_infos: GameState::init_draw_infos(),
        controlling: 0,
    };
    state.init_vertex_buffers(renderer);
    Ok(Box::leak(Box::new(state)))
}

fn main() {
    gfx_2020::main_loop::<gfx_backend_vulkan::Backend, _, _>(&render_config(), game_state_init_fn);
}
