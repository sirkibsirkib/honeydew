pub mod rendering;
pub mod room;

use {
    crate::{bit_set::INDICES, prelude::*, rng::Rng, wrap_fields::WrapVec2},
    gfx_2020::{gfx_hal::Backend, winit::event::ElementState, *},
    room::{Coord, Room, ROOM_DIMS},
};

pub const PLAYER_SIZE: Vec2 = Vec2 { x: 0.5, y: 0.5 };
pub const TELEPORTER_SIZE: Vec2 = Vec2 { x: 0.3, y: 0.3 };
pub const UP_WALL_SIZE: Vec2 = Vec2 { x: 1.0, y: 0.16 };

// allows an upper bound for renderer's instance buffers
pub const MAX_TELEPORTERS: u32 = INDICES as u32 / 64;
pub const MAX_PLAYERS: u32 = 32;
pub const MAX_WALLS: u32 = INDICES as u32 * 2;
/////////////////////////////////
struct Rect {
    center: Vec2,
    size: Vec2,
}
pub enum Net {
    Server { rng: Rng },
    Client {},
}
pub struct GameState {
    pub world: World,
    // controlling
    pub controlling: usize,
    pub pressing_state: PressingState,
    // rendering
    pub tex_id: TexId,
    pub draw_infos: [DrawInfo; 4], // four replicas of all instances to pan the maze indefinitely
    // network
    pub net: Net,
}
pub struct World {
    pub room: Room,
    pub players: Vec<Player>,
    pub teleporters: Vec<WrapVec2>,
}

#[derive(Debug)]
pub struct Player {
    pub pos: WrapVec2,
    pub vel: EnumMap<Dim, Option<Sign>>,
}
#[derive(Default, Debug)]
pub struct PressingState {
    map: EnumMap<Dim, AxisPressingState>,
}
#[derive(Copy, Clone, Debug)]
struct AxisPressingState {
    map: EnumMap<Sign, ElementState>,
}

impl Default for AxisPressingState {
    fn default() -> Self {
        Self {
            map: enum_map::enum_map! {
                Negative => ElementState::Released,
                Positive => ElementState::Released,
            },
        }
    }
}
impl Rect {
    fn contains(&self, pt: Vec2) -> bool {
        const GRACE_DISTANCE: f32 = 0.001;
        Dim::iter_domain().map(Dim::vec_index).all(|idx| {
            let pair = [pt[idx], self.center[idx]];
            modulo_distance(pair, ROOM_DIMS[idx] as f32) < self.size[idx] - GRACE_DISTANCE
        })
    }
    fn correct_point_collider(&self, pt: &mut Vec2) -> bool {
        if !self.contains(*pt) {
            return false;
        }
        let (idx, correction) = Dim::iter_domain()
            .map(|dim| {
                let idx = dim.vec_index();
                let correct_up = modulo_difference(
                    [self.center[idx] - self.size[idx] + ROOM_DIMS[idx] as f32, pt[idx]],
                    ROOM_DIMS[idx] as f32,
                );
                let correct_down = modulo_difference(
                    [self.center[idx] + self.size[idx], pt[idx]],
                    ROOM_DIMS[idx] as f32,
                );
                let min_correction =
                    if correct_up.abs() < correct_down.abs() { correct_up } else { correct_down };
                (idx, min_correction)
            })
            .min_by_key(|(_, correction)| OrderedFloat(correction.abs()))
            .unwrap();
        pt[idx] = (pt[idx] + correction + ROOM_DIMS[idx] as f32) % ROOM_DIMS[idx] as f32;
        true
    }
}
impl World {
    pub fn random_free_space(&self, rng: &mut Rng) -> Vec2 {
        const MIN_DIST: u32 = CELL_SIZE[X];
        loop {
            let new = Coord::random(rng).into_vec2_center();
            let mut pos_iter =
                self.teleporters.iter().copied().chain(self.players.iter().map(|p| p.pos));
            if pos_iter.all(|pos| pos.distance_squared(new) >= MIN_DIST) {
                return new;
            }
        }
    }
    fn move_and_collide<B: Backend>(&mut self, net: &mut Net, renderer: &mut Renderer<B>) {
        // update player positions wrt. movement
        for player in &mut self.players {
            // println!("{:?}", player.pos);
            for dim in Dim::iter_domain() {
                if let Some(sign) = player.vel[dim] {
                    player.pos[dim.vec_index()] += sign * 0.05;
                }
            }
        }

        // correct player positions wrt. player<->player collisions
        for [a, b] in iter_pairs_mut(&mut self.players) {
            Rect { center: b.pos, size: PLAYER_SIZE }.correct_point_collider(&mut a.pos);
        }

        // teleporter <-> colliders

        if let Net::Server { rng, .. } = net {
            for i in 0..self.players.len() {
                let player_pos = self.players[i].pos;
                for j in 0..self.teleporters.len() {
                    let teleporter = self.teleporters[j];
                    let rect =
                        Rect { center: teleporter, size: (PLAYER_SIZE + TELEPORTER_SIZE) / 2. };
                    if rect.contains(player_pos) {
                        self.players[i].pos = self.random_free_space(rng);
                        self.teleporters[j] = self.random_free_space(rng);
                    }
                }
            }
        }

        for player in &mut self.players {
            // 'correct_loop: loop {
            // wrap player positions
            GameState::wrap_pos(&mut player.pos);
            println!("at {:?}", player.pos);

            // correct position wrt. player<->wall collisions
            for dim in Dim::iter_domain() {
                let four_around = Coord::check_for_collisions_at(dim, player.pos);
                for (i, check_at) in four_around.enumerate() {
                    let collided = if self.room.wall_sets[dim].contains(check_at.into()) {
                        let rect = Rect {
                            center: GameState::wall_pos(check_at, dim),
                            size: GameState::wall_min_dists(dim),
                        };
                        let collided = rect.correct_point_collider(&mut player.pos);
                        if collided {
                            println!("COLLIDED");
                        }
                        true
                        // if collided {
                        //     continue 'correct_loop;
                        // }
                    } else {
                        false
                    };

                    let mut size = Vec2::from([0.2; 2]);
                    if collided {
                        size[dim.vec_index()] = 0.6;
                    }
                    renderer.write_vertex_buffer(
                        1 + i as u32 + if let X = dim { 4 } else { 0 },
                        Some(
                            Mat4::from_translation(check_at.into_vec2_center().extend(0.))
                                * Mat4::from_scale(size.extend(1.)),
                        ),
                    );
                }
            }
        }
    }
}
impl GameState {
    pub fn wrap_pos(pos: &mut Vec2) {
        const BOUND: Vec2 = Vec2 { x: ROOM_DIMS[0] as f32, y: ROOM_DIMS[1] as f32 };
        for idx in Dim::iter_domain().map(Dim::vec_index) {
            let value = &mut pos[idx];
            let bound = BOUND[idx];
            if *value < 0. {
                *value += bound;
            } else if bound < *value {
                *value -= bound;
            }
        }
    }
    fn wall_min_dists(dim: Dim) -> Vec2 {
        (Self::wall_size(dim) + PLAYER_SIZE) * 0.5
    }
    pub fn wall_size(dim: Dim) -> Vec2 {
        match dim {
            X => UP_WALL_SIZE,
            Y => UP_WALL_SIZE.yx(),
        }
    }
    pub fn wall_pos(coord: Coord, dim: Dim) -> Vec2 {
        // e.g. Hdimzontal wall at Coord[0,0] has pos [0.5, 0.0]
        let mut pos = coord.into_vec2_corner();
        pos[dim.vec_index()] += 0.5;
        pos
    }
    fn update_move_key(&mut self, dir: Direction, state: ElementState) {
        let dim = dir.dim();
        self.pressing_state.map[dim].map[dir.sign()] = state;
        self.world.players[self.controlling].vel[dim] = self.pressing_state.map[dim].solo_pressed();
    }
}

impl DrivesMainLoop for GameState {
    fn render<B: Backend>(&mut self, _: &mut Renderer<B>) -> ProceedWith<(usize, &[DrawInfo])> {
        Ok(self.get_draw_args())
    }

    fn update<B: Backend>(&mut self, renderer: &mut Renderer<B>) -> Proceed {
        self.world.move_and_collide(&mut self.net, renderer);
        self.update_vertex_buffers(renderer);
        self.update_view_transforms();
        Ok(())
    }

    fn handle_event<B: Backend>(
        &mut self,
        _renderer: &mut Renderer<B>,
        event: winit::event::Event<()>,
    ) -> Proceed {
        use winit::event::{
            Event as Ev, KeyboardInput as Ki, VirtualKeyCode as Vkc, WindowEvent as We,
        };
        match event {
            Ev::WindowEvent { event: We::CloseRequested, .. } => return Err(HaltLoop),
            Ev::WindowEvent { event: We::KeyboardInput { input, .. }, .. } => {
                // ok
                match input {
                    Ki { virtual_keycode: Some(Vkc::Escape), .. } => return Err(HaltLoop),
                    Ki { virtual_keycode: Some(Vkc::W), state, .. } => {
                        self.update_move_key(Up, state)
                    }
                    Ki { virtual_keycode: Some(Vkc::A), state, .. } => {
                        self.update_move_key(Left, state)
                    }
                    Ki { virtual_keycode: Some(Vkc::S), state, .. } => {
                        self.update_move_key(Down, state)
                    }
                    Ki { virtual_keycode: Some(Vkc::D), state, .. } => {
                        self.update_move_key(Right, state)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl AxisPressingState {
    fn solo_pressed(self) -> Option<Sign> {
        use ElementState::*;
        match [self.map[Negative], self.map[Positive]] {
            [Pressed, Pressed] | [Released, Released] => None,
            [Pressed, Released] => Some(Negative),
            [Released, Pressed] => Some(Positive),
        }
    }
}
