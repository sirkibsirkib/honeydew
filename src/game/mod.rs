pub mod bit_set;
pub mod rendering;
pub mod room;

use {
    crate::{basic::*, rng::Rng},
    bit_set::Coord,
    gfx_2020::{
        gfx_hal::Backend,
        winit::event::{ElementState, VirtualKeyCode},
        *,
    },
    room::Room,
};

pub const PLAYER_SIZE: Vec2 = Vec2 { x: 0.5, y: 0.5 };
pub const UP_WALL_SIZE: Vec2 = Vec2 { x: 1.0, y: 0.16 };

// allows an upper bound for renderer's instance buffers
pub const MAX_TELEPORTERS: u32 = bit_set::INDICES as u32 / 16;
pub const MAX_PLAYERS: u32 = 32;
pub const MAX_WALLS: u32 = bit_set::INDICES as u32 * 2;
/////////////////////////////////

pub struct GameState {
    // game world
    pub room: Room,
    pub players: Vec<Player>,
    pub teleporters: HashSet<Coord>,
    // controlling
    pub controlling: usize,
    pub pressing_state: PressingState,
    // rendering
    pub tex_id: TexId,
    pub draw_infos: [DrawInfo; 4], // four replicas of all instances to pan the maze indefinitely
}

#[derive(Debug)]
pub struct Player {
    pub pos: Vec2,
    pub vel: EnumMap<Orientation, Option<Sign>>,
}
#[derive(Default, Debug)]
pub struct PressingState {
    map: EnumMap<Orientation, AxisPressingState>,
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
impl GameState {
    pub fn unobstructed_center(&self, rng: &mut Rng) -> (Coord, Vec2) {
        loop {
            let coord = Coord::random(rng);
            let center = coord.into_vec2_center();
            const MIN_DIST: f32 = 2.;
            if self.players.iter().all(|p| p.pos.distance_squared(center) < MIN_DIST * MIN_DIST) {
                return (coord, center);
            }
        }
    }
    pub fn wrap_pos(pos: &mut Vec2) {
        const BOUND: Vec2 = Vec2 { x: bit_set::W as f32, y: bit_set::H as f32 };
        for idx in Orientation::iter_domain().map(Orientation::vec_index) {
            let value = &mut pos[idx];
            let bound = BOUND[idx];
            if *value < 0. {
                *value += bound;
            } else if bound < *value {
                *value -= bound;
            }
        }
    }
    fn wall_min_dists(ori: Orientation) -> Vec2 {
        (Self::wall_size(ori) + PLAYER_SIZE) * 0.5
    }
    pub fn wall_size(ori: Orientation) -> Vec2 {
        match ori {
            Horizontal => UP_WALL_SIZE,
            Vertical => UP_WALL_SIZE.yx(),
        }
    }
    pub fn wall_pos(coord: Coord, ori: Orientation) -> Vec2 {
        // e.g. Horizontal wall at Coord[0,0] has pos [0.5, 0.0]
        let mut pos = coord.into_vec2_corner();
        pos[ori.vec_index()] += 0.5;
        pos
    }
    fn collide_with(a_pos: &mut Vec2, b_pos: Vec2, min_dists: Vec2) -> bool {
        const MIN_DELTA: f32 = 0.01;
        let a_rel = *a_pos - b_pos; // position of A relative to position of B
        let colliding = Orientation::iter_domain()
            .map(Orientation::vec_index)
            .all(|idx| a_rel[idx].abs() + MIN_DELTA < min_dists[idx]);
        if !colliding {
            return false;
        }
        let (idx, correction) = Orientation::iter_domain()
            .map(|ori| {
                let idx = ori.vec_index();
                let a_rel = a_rel[idx];
                let min_dist = min_dists[idx];
                let a_corrected = if 0. < a_rel { min_dist } else { -min_dist };
                let correction = a_corrected - a_rel;
                (idx, correction)
            })
            .min_by_key(|(_, correction)| OrderedFloat(correction.abs()))
            .unwrap();
        a_pos[idx] += correction;
        true
    }
    fn move_players(&mut self) {
        // 1. update my velocity
        for ori in Orientation::iter_domain() {
            self.players[self.controlling].vel[ori] = self.pressing_state.map[ori].solo_pressed();
        }

        // update player positions wrt. movement
        for player in &mut self.players {
            for ori in Orientation::iter_domain() {
                if let Some(sign) = player.vel[ori] {
                    player.pos[ori.vec_index()] += sign * 0.05;
                }
            }
        }

        // correct player positions wrt. player<->player collisions
        for [a, b] in iter_pairs_mut(&mut self.players) {
            Self::collide_with(&mut a.pos, b.pos, PLAYER_SIZE);
        }

        for player in &mut self.players {
            // wrap player positions
            Self::wrap_pos(&mut player.pos);
            // resolve collisions
            const BOUND: Vec2 = Vec2 { x: bit_set::W as f32, y: bit_set::H as f32 };
            for idx in Orientation::iter_domain().map(Orientation::vec_index) {
                let value = &mut player.pos[idx];
                let bound = BOUND[idx];
                if *value < 0. {
                    *value += bound;
                } else if bound < *value {
                    *value -= bound;
                }
            }
            // correct position wrt. player<->wall collisions
            let at = Coord::from_vec2_rounded(player.pos);
            for ori in Orientation::iter_domain() {
                // Eg: check for horizontal walls in THIS cell, cell to the left, and cell to the right
                let check_at = [at.stepped(ori.sign(Negative)), at, at.stepped(ori.sign(Positive))];
                for &coord in check_at.iter() {
                    if self.room.wall_sets[ori].contains(coord.into()) {
                        let collided = Self::collide_with(
                            &mut player.pos,
                            Self::wall_pos(coord, ori),
                            Self::wall_min_dists(ori),
                        );
                        if collided {
                            Self::wrap_pos(&mut player.pos);
                        }
                    }
                }
            }
        }
    }
    fn pressing_state_update(&mut self, vkc: VirtualKeyCode, state: ElementState) -> bool {
        use VirtualKeyCode as Vkc;
        let (orientation, sign) = match vkc {
            Vkc::W => (Vertical, Negative),
            Vkc::A => (Horizontal, Negative),
            Vkc::S => (Vertical, Positive),
            Vkc::D => (Horizontal, Positive),
            _ => return false,
        };
        self.pressing_state.map[orientation].map[sign] = state;
        true
    }
}

impl DrivesMainLoop for GameState {
    fn render<B: Backend>(&mut self, _: &mut Renderer<B>) -> ProceedWith<(usize, &[DrawInfo])> {
        Ok((self.tex_id, &self.draw_infos))
    }

    fn update<B: Backend>(&mut self, renderer: &mut Renderer<B>) -> Proceed {
        self.move_players();
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
                    Ki { virtual_keycode: Some(vk), state, .. }
                        if self.pressing_state_update(vk, state) => {}
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
