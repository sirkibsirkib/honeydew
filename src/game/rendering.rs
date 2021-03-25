use {
    crate::{
        game::{
            GameState, MAX_PLAYERS, MAX_TELEPORTERS, MAX_WALLS, PLAYER_SIZE, TELEPORTER_SIZE,
            UP_WALL_SIZE,
        },
        prelude::*,
    },
    gfx_2020::{
        gfx_hal::{pso::Face, window::Extent2D, Backend},
        vert_coord_consts::UNIT_QUAD,
        DrawInfo, MaxBufferArgs, Renderer, RendererConfig, RendererInitConfig, TexId, TexScissor,
    },
};
pub const INSTANCE_RANGE_PLAYERS: Range<u32> = 0..MAX_PLAYERS;
pub const INSTANCE_RANGE_TELEPORTERS: Range<u32> =
    range_concat(INSTANCE_RANGE_PLAYERS, MAX_TELEPORTERS);
pub const INSTANCE_RANGE_WALLS: Range<u32> = range_concat(INSTANCE_RANGE_TELEPORTERS, MAX_WALLS);
pub const MAX_INSTANCES: u32 = INSTANCE_RANGE_WALLS.end;

pub const WRAP_DRAW: bool = false;

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
    pub fn get_draw_args(&self) -> (TexId, &[DrawInfo]) {
        let range = 0..if WRAP_DRAW { 4 } else { 1 };
        (self.tex_id, &self.draw_infos[range])
    }
    pub fn init_draw_infos() -> [DrawInfo; 4] {
        let new_draw_info = || DrawInfo {
            instance_range: 0..MAX_INSTANCES,
            view_transform: Mat4::identity(),
            vertex_range: 0..UNIT_QUAD.len() as u32,
        };
        [new_draw_info(), new_draw_info(), new_draw_info(), new_draw_info()]
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
        self.update_teleporter_transforms(renderer);
    }
    pub fn update_view_transforms(&mut self) {
        const SCALE: f32 = 1. / 16.;
        const SCALE_XY: Vec2 = Vec2 { x: SCALE, y: SCALE };
        let translations = {
            let mut s = self.world.players[self.controlling].pos.to_screen2();
            // shift pos s.t. we focus on the replica closest to the center
            if WRAP_DRAW {
                for value in s.as_mut() {
                    if *value < 0.5 {
                        *value += 0.5;
                    }
                }
            }
            [-s, -s + [0., 1.].into(), -s + [1., 0.].into(), -s + [1., 1.].into()]
        };
        for (draw_info, pos_vec2) in self.draw_infos.iter_mut().zip(translations.iter()) {
            draw_info.view_transform =
                Mat4::from_scale(SCALE_XY.extend(1.)) * Mat4::from_translation(pos_vec2.extend(0.))
        }
    }
    fn update_tri_verts<B: Backend>(renderer: &mut Renderer<B>) {
        renderer.write_vertex_buffer(0, UNIT_QUAD.iter().copied());
    }
    fn update_wall_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        let iter = self.world.room.iter_walls().map(move |(coord, dim)| {
            Mat4::from_translation(GameState::wall_pos(coord, dim).to_screen2().extend(0.))
                * Mat4::from_rotation_z(if let Y = dim { PI_F32 * -0.5 } else { 0. })
                * Mat4::from_scale(UP_WALL_SIZE.to_screen2().extend(1.))
        });
        renderer.write_vertex_buffer(INSTANCE_RANGE_WALLS.start, iter);
    }
    fn update_player_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        let iter = self.world.players.iter().map(move |player| {
            Mat4::from_translation(player.pos.to_screen2().extend(0.))
                * Mat4::from_scale(PLAYER_SIZE.to_screen2().extend(1.))
        });
        renderer.write_vertex_buffer(INSTANCE_RANGE_PLAYERS.start, iter);
    }
    fn update_teleporter_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        let iter = self.world.teleporters.iter().map(move |pos| {
            Mat4::from_translation(pos.to_screen2().extend(0.))
                * Mat4::from_scale(TELEPORTER_SIZE.to_screen2().extend(1.))
        });
        renderer.write_vertex_buffer(INSTANCE_RANGE_TELEPORTERS.start, iter);
    }
    fn update_tex_scissors<B: Backend>(&self, renderer: &mut Renderer<B>) {
        use std::iter::repeat;
        // teleporters
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_TELEPORTERS.start,
            repeat(scissor_for_tile_at([0, 1])).take(self.world.teleporters.len()),
        );
        // players
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_PLAYERS.start,
            repeat(scissor_for_tile_at([3, 0])).take(self.world.players.len()),
        );
        // walls
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_WALLS.start,
            repeat(scissor_for_tile_at([0, 0])).take(self.world.room.wall_count() as usize),
        );
    }
}
