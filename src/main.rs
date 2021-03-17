////////////////////////////
mod basic;
mod game;
mod rng;

use {
    crate::{
        basic::*,
        game::{bit_set::Coord, room::Room, GameState, Player},
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
            max_instances: 2048,
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
    room.ascii_print();
    let wall_count = room.wall_count();
    let instance_count = wall_count + 1;

    let tri_vert_iter = UNIT_QUAD.iter().copied();
    let wall_transform_iter = room.iter_walls().map(|(coord, orientation)| {
        let [x, y] = coord.xy();
        let [mut tx, mut ty] = [x as f32, y as f32];
        let (coord, rot) = match orientation {
            Orientation::Horizontal => (&mut ty, 0.), // up walls are moved UP and NOT rotated
            Orientation::Vertical => (&mut tx, std::f32::consts::PI * -0.5), // left walls are moved LEFT and are ARE rotated 90 degrees
        };
        *coord -= 0.5;
        Mat4::from_translation(Vec3::new(tx, ty, 0.))
            * Mat4::from_rotation_z(rot)
            * Mat4::from_scale(Vec3::new(1.0, 0.16, 1.0))
    });

    let wall_tex_scissor_iter = std::iter::repeat(TexScissor {
        top_left: [0., 0.],
        size: [6.0f32.recip(), 3.0f32.recip()],
    })
    .take(instance_count as usize);

    renderer.write_vertex_buffer(0, tri_vert_iter);
    renderer.write_vertex_buffer(0, wall_transform_iter);
    renderer.write_vertex_buffer(0, wall_tex_scissor_iter);

    let controlling = 0;
    let players = [Player {
        pos: {
            let [x, y] = Coord::random_tl_interior(&mut rng).xy();
            Vec3::new(x as f32, y as f32, 0.)
        },
        vel: Vec3::from([0.; 3]),
    }];
    let draw_infos = [DrawInfo {
        instance_range: 0..instance_count,
        view_transform: GameState::calc_view_transform(players[controlling].pos),
        vertex_range: 0..UNIT_QUAD.len() as u32,
    }];

    Ok(heap_leak(GameState {
        pressing_state: Default::default(),
        room,
        tex_id,
        draw_infos,
        players,
        controlling,
    }))
}

fn main() {
    gfx_2020::main_loop::<VulkanBackend, _, _>(&render_config(), game_state_init_fn);
}
