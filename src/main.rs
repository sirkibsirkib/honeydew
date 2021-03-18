////////////////////////////
mod basic;
mod game;
mod rng;

use crate::bit_set::BitSet;
use {
    crate::{
        basic::*,
        game::{
            bit_set::{self, Coord},
            room::Room,
            GameState, Player,
        },
        rng::Rng,
    },
    gfx_2020::{
        gfx_hal::{pso::Face, window::Extent2D, Backend},
        vert_coord_consts::UNIT_QUAD,
        DrawInfo, Vec3, *,
    },
    gfx_backend_vulkan::Backend as VulkanBackend,
};
/////////////////////////////////

/////////////////////////////

fn render_config() -> RendererConfig<'static> {
    RendererConfig {
        init: RendererInitConfig {
            window_dims: Extent2D { width: 900, height: 900 },
            cull_face: Face::NONE,
            ..Default::default()
        },
        max_buffer_args: MaxBufferArgs {
            max_tri_verts: UNIT_QUAD.len() as u32,
            max_instances: bit_set::INDICES as u32 * 2 + game::PLAYER_CAP,
        },
        ..Default::default()
    }
}

fn heap_leak<T>(t: T) -> &'static mut T {
    Box::leak(Box::new(t))
}

pub(crate) fn game_state_init_fn<B: Backend>(
    renderer: &mut Renderer<B>,
) -> ProceedWith<&'static mut GameState> {
    let texture = gfx_2020::load_texture_from_path("./src/data/faces.png").unwrap();
    let tex_id = renderer.load_texture(&texture);
    let mut rng = Rng::new(Some(0));
    let room = Room::new(&mut rng);
    // let room = Room {
    //     wall_sets: enum_map::enum_map! {
    //         Horizontal => BitSet::default(),
    //         Vertical => Some(Coord::new([5, 5])).into_iter().collect(),
    //     },
    // };
    room.ascii_print();
    let wall_count = room.wall_count();
    let player_count = 1;
    assert!(player_count <= game::PLAYER_CAP);
    let instance_count = wall_count + player_count;

    let tri_vert_iter = UNIT_QUAD.iter().copied();
    let wall_transform_iter = room.iter_walls().map(|(coord, ori)| {
        Mat4::from_translation(GameState::wall_pos(coord, ori).extend(0.))
            * Mat4::from_rotation_z(if let Vertical = ori { PI_F32 * -0.5 } else { 0. })
            * Mat4::from_scale(game::UP_WALL_SIZE.extend(1.))
    });

    fn scissor_for_tile_at([x, y]: [u16; 2]) -> TexScissor {
        const TILE_SIZE: Vec2 = Vec2 { x: 1. / 6., y: 1. / 3. };
        TexScissor {
            top_left: Vec2::new(TILE_SIZE[0] * x as f32, TILE_SIZE[1] * y as f32),
            size: TILE_SIZE,
        }
    }
    let wall_tex_scissor_iter =
        std::iter::repeat(scissor_for_tile_at([0, 0])).take(wall_count as usize);
    let player_tex_scissor_iter =
        std::iter::repeat(scissor_for_tile_at([3, 0])).take(player_count as usize);

    renderer.write_vertex_buffer(0, tri_vert_iter);
    renderer.write_vertex_buffer(0, wall_transform_iter);
    renderer.write_vertex_buffer(0, wall_tex_scissor_iter.chain(player_tex_scissor_iter));

    let controlling = 0;
    let players = {
        let mut players = Vec::<Player>::with_capacity(player_count as usize);
        for _ in 0..player_count {
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
    let new_draw_info = || DrawInfo {
        instance_range: 0..instance_count,
        view_transform: Mat4::identity(),
        vertex_range: 0..UNIT_QUAD.len() as u32,
    };
    let draw_infos = [new_draw_info(), new_draw_info(), new_draw_info(), new_draw_info()];
    let mut state = GameState {
        player_instances_start: wall_count,
        pressing_state: Default::default(),
        room,
        tex_id,
        draw_infos,
        players,
        controlling,
    };
    state.update_player_transforms(renderer);
    state.update_view_transforms();
    Ok(heap_leak(state))
}

fn main() {
    gfx_2020::main_loop::<VulkanBackend, _, _>(&render_config(), game_state_init_fn);
}
