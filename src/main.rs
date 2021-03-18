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
    gfx_backend_vulkan::Backend as VulkanBackend,
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
    let players = {
        let mut players = Vec::<Player>::with_capacity(PLAYER_COUNT as usize);
        for _ in 0..PLAYER_COUNT {
            let pos = 'pos_guess: loop {
                let pos: Vec2 = Coord::random(&mut rng).into();
                for player in players.iter() {
                    if player.pos.distance(pos) < 2. {
                        continue 'pos_guess;
                    }
                }
                break 'pos_guess pos;
            };
            players.push(Player { pos, vel: Default::default() });
        }
        players
    };
    let state = GameState {
        pressing_state: Default::default(),
        teleporters: Default::default(),
        room,
        tex_id,
        draw_infos: GameState::init_draw_infos(),
        players,
        controlling,
    };
    state.init_vertex_buffers(renderer);
    Ok(Box::leak(Box::new(state)))
}

fn main() {
    gfx_2020::main_loop::<VulkanBackend, _, _>(&render_config(), game_state_init_fn);
}
