////////////////////////////
mod basic;
mod bit_set;
mod game;
mod point;
mod rng;

use {
    crate::{
        basic::*,
        game::{
            rendering::render_config, room::Room, GameState, Net, Player, World, MAX_TELEPORTERS,
        },
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
    let mut rng = Rng::new(Some(1));
    let room = Room::new(&mut rng);
    room.ascii_print();
    const PLAYER_COUNT: u32 = 1;
    let controlling = 0;
    let mut world = World {
        players: Vec::with_capacity(PLAYER_COUNT as usize),
        teleporters: Default::default(),
        room,
    };
    for _ in 0..PLAYER_COUNT {
        let pos = world.random_free_space(&mut rng);
        println!("player @ {:?}", pos);
        let player = Player { pos, vel: Default::default() };
        world.players.push(player);
    }
    for _ in 0..MAX_TELEPORTERS {
        let pos = world.random_free_space(&mut rng);
        println!("teleporter @ {:?}", pos);
        world.teleporters.push(pos);
    }
    let state = GameState {
        net: Net::Server { rng },
        world,
        pressing_state: Default::default(),
        tex_id,
        draw_infos: GameState::init_draw_infos(),
        controlling,
    };
    state.init_vertex_buffers(renderer);
    Ok(Box::leak(Box::new(state)))
}

fn main() {
    use point::*;
    let mut pt = Point::ZERO;
    for _ in 0..10 {
        pt[Horizontal] += 1;
        println!("{:?}", pt);
        pt = -pt;
        println!("{:?}", pt);
    }
    // for i in 0..5 {
    //     for j in 0..5 {
    //         let a = i as f32;
    //         let b = j as f32;
    //         let ans = modulo_difference([a, b], 5.);
    //         println!(
    //             "{:?}\t({} + {}) % 5. == {}\t{}",
    //             [a, b],
    //             b,
    //             ans,
    //             a,
    //             if ans.is_nan() {
    //                 "NAN"
    //             } else if (b + ans) % 5. == a {
    //                 "YES"
    //             } else {
    //                 "NO"
    //             }
    //         );
    //     }
    // }
    return;
    gfx_2020::main_loop::<gfx_backend_vulkan::Backend, _, _>(&render_config(), game_state_init_fn);
}
