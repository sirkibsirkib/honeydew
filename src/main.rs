////////////////////////////
mod basic;
mod game;
mod rng;

use {
    crate::{
        basic::*,
        game::{bit_set::Coord, rendering::render_config, room::Room, GameState, Player},
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
    let mut rng = Rng::new(Some(0));
    let room = Room::new(&mut rng);
    room.ascii_print();
    const PLAYER_COUNT: u32 = 1;
    let controlling = 0;
    let mut state = GameState {
        pressing_state: Default::default(),
        teleporters: Default::default(),
        room,
        tex_id,
        draw_infos: GameState::init_draw_infos(),
        players: Vec::with_capacity(PLAYER_COUNT as usize),
        controlling,
    };
    for _ in 0..PLAYER_COUNT {
        let (_coord, pos) = state.unobstructed_center(&mut rng);
        println!("{:?}", pos);
        let player = Player { pos, vel: Default::default() };
        state.players.push(player);
    }
    state.init_vertex_buffers(renderer);
    Ok(Box::leak(Box::new(state)))
}

fn main() {
    gfx_2020::main_loop::<gfx_backend_vulkan::Backend, _, _>(&render_config(), game_state_init_fn);
}
