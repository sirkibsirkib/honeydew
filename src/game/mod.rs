pub mod bit_set;
pub mod room;

use {
    crate::basic::*,
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
pub const PLAYER_CAP: u32 = 32;
/////////////////////////////////

pub struct GameState {
    pub room: Room,
    pub controlling: usize,
    pub player_instances_start: u32,
    pub players: Vec<Player>,
    pub pressing_state: PressingState,
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
#[derive(Debug, Clone)]
struct Rect {
    center: Vec2,
    size: Vec2,
}
/////////////////////////////////

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
        let mut pos: Vec2 = coord.into();
        pos[ori.vec_index()] -= 0.5;
        pos
    }
    fn collide_with(a_pos: &mut Vec2, b_pos: Vec2, min_dists: Vec2) {
        let a_rel = *a_pos - b_pos; // position of A relative to position of B
        let no_collision = Orientation::iter_domain()
            .map(Orientation::vec_index)
            .any(|idx| min_dists[idx] <= a_rel[idx].abs());
        if no_collision {
            return;
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
    }
    fn update_player_data(&mut self) {
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
            Self::collide_with(&mut a.pos, b.pos, PLAYER_SIZE)
        }

        for player in &mut self.players {
            // wrap player positions
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
            for coord in Into::<Coord>::into(player.pos).nine_grid_iter() {
                for ori in Orientation::iter_domain() {
                    if self.room.wall_sets[ori].contains(coord.into()) {
                        Self::collide_with(
                            &mut player.pos,
                            Self::wall_pos(coord, ori),
                            Self::wall_min_dists(ori),
                        )
                    }
                }
            }
        }
    }
    pub(crate) fn update_player_transforms<B: Backend>(&mut self, renderer: &mut Renderer<B>) {
        renderer.write_vertex_buffer(
            self.player_instances_start,
            self.players.iter().map(|player| {
                Mat4::from_translation(player.pos.extend(0.))
                    * Mat4::from_scale(PLAYER_SIZE.extend(1.))
            }),
        );
    }
    pub(crate) fn update_view_transforms(&mut self) {
        const SCALE: f32 = 1. / 16.;
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
        self.update_player_data();
        self.update_player_transforms(renderer);
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
