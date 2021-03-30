use {
    crate::{
        game::{
            room::{CELL_COUNTS, ROOM_SIZE},
            GameState, MyDoor, MAX_WALLS, NUM_DRAW_INFOS, NUM_MY_DOORS, NUM_PLAYERS,
            NUM_TELEPORTERS, PLAYER_SIZE, TELEPORTER_SIZE, WALL_SIZE,
        },
        prelude::*,
    },
    gfx_2020::{
        gfx_hal::{pso::Face, window::Extent2D, Backend},
        vert_coord_consts::UNIT_QUAD,
        DrawInfo, MaxBufferArgs, Renderer, RendererConfig, RendererInitConfig, TexId, TexScissor,
    },
};

pub const INSTANCE_RANGE_PLAYERS: Range<u32> = 0..NUM_PLAYERS;
pub const INSTANCE_RANGE_TELEPORTERS: Range<u32> =
    range_concat(INSTANCE_RANGE_PLAYERS, NUM_TELEPORTERS);
pub const INSTANCE_RANGE_MY_DOORS: Range<u32> =
    range_concat(INSTANCE_RANGE_TELEPORTERS, NUM_MY_DOORS);
pub const INSTANCE_RANGE_WALLS: Range<u32> = range_concat(INSTANCE_RANGE_MY_DOORS, MAX_WALLS);
pub const MAX_INSTANCES: u32 = INSTANCE_RANGE_WALLS.end;

// for debugging. true for release.
pub const ENABLE_WRAP_DRAW: bool = true;

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
    const TILE_SIZE: Vec2 = Vec2 { x: 1. / 5., y: 1. / 2. };
    TexScissor {
        top_left: Vec2::new(TILE_SIZE[0] * x as f32, TILE_SIZE[1] * y as f32),
        size: TILE_SIZE,
    }
}

impl GameState {
    pub fn get_draw_args(&self) -> (TexId, ClearColor, &[DrawInfo]) {
        let range = 0..if ENABLE_WRAP_DRAW { NUM_DRAW_INFOS } else { 1 };
        (self.tex_id, ClearColor { float32: [0.5, 0.5, 0.5, 1.0] }, &self.draw_infos[range])
    }
    pub fn init_draw_infos() -> [DrawInfo; NUM_DRAW_INFOS] {
        let new_draw_info = || DrawInfo {
            instance_range: 0..MAX_INSTANCES,
            view_transform: Mat4::identity(),
            vertex_range: 0..UNIT_QUAD.len() as u32,
        };
        [new_draw_info(), new_draw_info(), new_draw_info(), new_draw_info()]
    }
    pub fn init_vertex_buffers<B: Backend>(&mut self, renderer: &mut Renderer<B>) {
        // called ONCE as game starts
        Self::update_tri_verts(renderer);
        self.update_tex_scissors(renderer);
        self.update_wall_transforms(renderer);
        self.update_my_door_transforms(renderer);
        self.update_vertex_buffers(renderer);
    }
    pub fn update_vertex_buffers<B: Backend>(&mut self, renderer: &mut Renderer<B>) {
        // called once per update tick
        self.update_player_transforms(renderer);
        self.update_teleporter_transforms(renderer);
        self.randomize_teleporter_tex_scissors(renderer);
        self.update_my_door_transforms(renderer);
    }
    pub fn update_view_transforms(&mut self) {
        const ZOOM_OUT: f32 = 4.;
        const SCALE_XY: Vec2 = Vec2 {
            x: CELL_COUNTS.arr[0] as f32 / ZOOM_OUT,
            y: CELL_COUNTS.arr[1] as f32 / ZOOM_OUT,
        };
        let translations = {
            let mut s = self.world.entities.players[self.controlling].pos.to_screen2();
            if ENABLE_WRAP_DRAW {
                for idx in 0..2 {
                    if s[idx] < 0.5 {
                        s[idx] += 1.;
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
            Mat4::from_translation(GameState::wall_pos(coord, dim).to_screen2().extend(0.5))
                * Mat4::from_scale(WALL_SIZE[dim].to_screen2().extend(1.))
        });
        renderer.write_vertex_buffer(INSTANCE_RANGE_WALLS.start, iter);
    }
    fn update_my_door_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        for my_door_idx in self.my_doors_just_moved.into_iter() {
            let MyDoor { coord, dim } = self.my_doors[my_door_idx];
            let t = Mat4::from_translation(GameState::wall_pos(coord, dim).to_screen2().extend(0.))
                * Mat4::from_scale(WALL_SIZE[dim].to_screen2().extend(1.));
            renderer.write_vertex_buffer(INSTANCE_RANGE_MY_DOORS.start, std::iter::once(t));
        }
    }
    fn update_player_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        let iter = self.world.entities.players.iter().map(move |player| {
            Mat4::from_translation(player.pos.to_screen2().extend(0.))
                * Mat4::from_scale(PLAYER_SIZE.to_screen2().extend(1.))
        });
        renderer.write_vertex_buffer(INSTANCE_RANGE_PLAYERS.start, iter);
    }
    fn update_teleporter_transforms<B: Backend>(&self, renderer: &mut Renderer<B>) {
        let iter = self.world.entities.teleporters.iter().map(move |pos| {
            Mat4::from_translation(pos.to_screen2().extend(0.))
                * Mat4::from_scale(TELEPORTER_SIZE.to_screen2().extend(1.))
        });
        renderer.write_vertex_buffer(INSTANCE_RANGE_TELEPORTERS.start, iter);
    }
    fn randomize_teleporter_tex_scissors<B: Backend>(&mut self, renderer: &mut Renderer<B>) {
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_TELEPORTERS.start,
            (0..self.world.entities.teleporters.len()).map(|_| {
                scissor_for_tile_at([
                    (self.local_rng.gen_bits(2) + self.local_rng.gen_bits(1)) as u16, // generates in 0..5 with bias toward 1..4
                    1,
                ])
            }),
        );
    }
    fn update_tex_scissors<B: Backend>(&mut self, renderer: &mut Renderer<B>) {
        use std::iter::repeat;
        // teleporters
        self.randomize_teleporter_tex_scissors(renderer);
        // players
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_PLAYERS.start,
            (0..3).map(|x| scissor_for_tile_at([x + 2, 0])),
        );
        // my doors
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_MY_DOORS.start,
            repeat(scissor_for_tile_at([1, 0])).take(NUM_MY_DOORS as usize),
        );
        // walls
        renderer.write_vertex_buffer(
            INSTANCE_RANGE_WALLS.start,
            repeat(scissor_for_tile_at([0, 0])).take(self.world.room.wall_count() as usize),
        );
    }
}

impl Pos {
    pub fn to_screen2(self) -> Vec2 {
        self.map(Into::<u16>::into).to_screen2()
    }
}
impl Size {
    pub fn to_screen2(self) -> Vec2 {
        let f = move |dim| self[dim] as f32 / ROOM_SIZE[dim] as f32;
        Vec2 { x: f(X), y: f(Y) }
    }
}
