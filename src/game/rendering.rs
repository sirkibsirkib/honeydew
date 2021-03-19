use {
    crate::{
        basic::*,
        game::{
            bit_set, GameState, MAX_PLAYERS, MAX_TELEPORTERS, MAX_WALLS, PLAYER_SIZE, UP_WALL_SIZE,
        },
    },
    gfx_2020::{
        gfx_hal::{pso::Face, window::Extent2D, Backend},
        vert_coord_consts::UNIT_QUAD,
        DrawInfo, MaxBufferArgs, Renderer, RendererConfig, RendererInitConfig, TexScissor,
    },
};
pub const INSTANCE_RANGE_PLAYERS: Range<u32> = 0..MAX_PLAYERS;
pub const INSTANCE_RANGE_TELEPORTERS: Range<u32> =
    range_concat(INSTANCE_RANGE_PLAYERS, MAX_TELEPORTERS);
pub const INSTANCE_RANGE_WALLS: Range<u32> = range_concat(INSTANCE_RANGE_TELEPORTERS, MAX_WALLS);
pub const MAX_INSTANCES: u32 = INSTANCE_RANGE_WALLS.end;

/////////////////////////////////
pub fn render_config() -> RendererConfig<'static> {
    RendererConfig {
        init: RendererInitConfig {
            window_dims: Extent2D { width: 900, height: 900 },
            cull_face: Face::NONE,
            ..Default::default()
        },
        max_buffer_args: MaxBufferArgs {
            max_tri_verts: UNIT_QUAD.len() as u32,
            max_instances: MAX_INSTANCES,
        },
        ..Default::default()
    }
}
const fn range_concat(r: Range<u32>, len: u32) -> Range<u32> {
    r.end..r.end + len
}

fn scissor_for_tile_at([x, y]: [u16; 2]) -> TexScissor {
    const TILE_SIZE: Vec2 = Vec2 { x: 1. / 6., y: 1. / 3. };
    TexScissor {
        top_left: Vec2::new(TILE_SIZE[0] * x as f32, TILE_SIZE[1] * y as f32),
        size: TILE_SIZE,
    }
}
impl GameState {
    pub fn init_draw_infos() -> [DrawInfo; 4] {
        let new_draw_info = || DrawInfo {
            instance_range: 0..MAX_INSTANCES,
            view_transform: Mat4::identity(),
            vertex_range: 0..UNIT_QUAD.len() as u32,
        };
        [new_draw_info(), new_draw_info(), new_draw_info(), new_draw_info()]
    }
    fn calc_wall_transforms(&self) -> impl Iterator<Item = Mat4> + '_ {
        self.room.iter_walls().map(move |(coord, ori)| {
            Mat4::from_translation(GameState::wall_pos(coord, ori).extend(0.))
                * Mat4::from_rotation_z(if let Vertical = ori { PI_F32 * -0.5 } else { 0. })
                * Mat4::from_scale(UP_WALL_SIZE.extend(1.))
        })
    }
    fn calc_player_transforms(&self) -> impl Iterator<Item = Mat4> + '_ {
        self.players.iter().map(move |player| {
            Mat4::from_translation(player.pos.extend(0.)) * Mat4::from_scale(PLAYER_SIZE.extend(1.))
        })
    }
    pub fn init_vertex_buffers<B: Backend>(&self, renderer: &mut Renderer<B>) {
        // called ONCE as game starts
        Self::update_tri_verts(renderer);
        self.update_tex_scissors(renderer);
        self.update_wall_transforms(renderer);
        self.update_vertex_buffers(renderer);
    }
    pub fn update_vertex_buffers<B: Backend>(&self, renderer: &mut Renderer<B>) {
        // called once per update tick
        self.update_player_transforms(renderer);
    }
    pub fn update_view_transforms(&mut self) {
        const SCALE: f32 = 1. / 6.;
        const SCALE_XY: Vec2 = Vec2 { x: SCALE, y: SCALE };
        let translations = {
            const W: f32 = bit_set::W as f32;
            const H: f32 = bit_set::H as f32;
            let mut base = self.players[self.controlling].pos;
            // by default, we view the TOPLEFT copy!
            if base[0] < W * 0.5 {
                // shift to RIGHT view
                base[0] += W;
            }
            if base[1] < H * 0.5 {
                // shift to BOTTOM view
                base[1] += H;
            }
            [-base, Vec2::new(W, 0.) - base, Vec2::new(0., H) - base, Vec2::new(W, H) - base]
        };
        for (draw_info, translation) in self.draw_infos.iter_mut().zip(translations.iter()) {
            draw_info.view_transform = Mat4::from_scale(SCALE_XY.extend(1.))
                * Mat4::from_translation(translation.extend(0.))
        }
    }
    fn update_tri_verts<B: Backend>(renderer: &mut Renderer<B>) {
        renderer.write_vertex_buffer(0, UNIT_QUAD.iter().copied());
    }
    fn update_wall_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        renderer.write_vertex_buffer(INSTANCE_RANGE_WALLS.start, self.calc_wall_transforms());
    }
    fn update_player_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        renderer.write_vertex_buffer(INSTANCE_RANGE_PLAYERS.start, self.calc_player_transforms());
    }
    fn update_tex_scissors<B: Backend>(&self, renderer: &mut Renderer<B>) {
        use std::iter::repeat;
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_WALLS.start,
            repeat(Self::wall_tex_scissor()).take(self.room.wall_count() as usize),
        );
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_PLAYERS.start,
            repeat(Self::player_tex_scissor()).take(self.players.len()),
        );
    }
}
impl GameState {
    pub fn wall_tex_scissor() -> TexScissor {
        scissor_for_tile_at([0, 0])
    }
    pub fn player_tex_scissor() -> TexScissor {
        scissor_for_tile_at([3, 0])
    }
}