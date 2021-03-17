pub mod bit_set;
pub mod room;

use {
    crate::basic::*,
    gfx_2020::{
        gfx_hal::Backend,
        winit::event::{ElementState, VirtualKeyCode},
        *,
    },
    room::Room,
};

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
    pub pos: Vec3,
    pub vel: Vec3,
}
#[derive(Default, Debug)]
pub struct PressingState {
    map: enum_map::EnumMap<Orientation, AxisPressingState>,
}
#[derive(Copy, Clone, Debug)]
struct AxisPressingState {
    map: enum_map::EnumMap<Sign, ElementState>,
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
    fn update_player_positions(&mut self) {
        fn wrap_value(value: &mut f32, bound: f32) {
            if *value < 0. {
                *value += bound;
            } else if bound < *value {
                *value -= bound;
            }
        };
        for player in self.players.iter_mut() {
            player.pos += player.vel;
            wrap_value(&mut player.pos[0], bit_set::W as f32);
            wrap_value(&mut player.pos[1], bit_set::H as f32);
        }
    }
    pub(crate) fn update_player_transforms<B: Backend>(&mut self, renderer: &mut Renderer<B>) {
        renderer.write_vertex_buffer(
            self.player_instances_start,
            self.players.iter().map(|player| {
                Mat4::from_translation(player.pos) * Mat4::from_scale(Vec3::new(0.7, 0.7, 1.))
            }),
        );
    }
    pub(crate) fn update_view_transforms(&mut self) {
        const SCALE_XY: [f32; 2] = [1. / 6.; 2];
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
            base = -base;
            [
                base,
                base + Vec3::new(W, 0., 0.),
                base + Vec3::new(0., H, 0.),
                base + Vec3::new(W, H, 0.),
            ]
        };
        for (draw_info, translation) in self.draw_infos.iter_mut().zip(translations.iter()) {
            draw_info.view_transform = Mat4::from_scale(Vec3::new(SCALE_XY[0], SCALE_XY[1], 1.0))
                * Mat4::from_translation(*translation)
        }
    }
    fn pressing_state_update(&mut self, vkc: VirtualKeyCode, state: ElementState) -> bool {
        use VirtualKeyCode as Vkc;
        const SPEED: f32 = 0.05;

        let (orientation, sign) = match vkc {
            Vkc::W => (Vertical, Negative),
            Vkc::A => (Horizontal, Negative),
            Vkc::S => (Vertical, Positive),
            Vkc::D => (Horizontal, Positive),
            _ => return false,
        };
        let value = &mut self.pressing_state.map[orientation].map[sign];
        if *value != state {
            *value = state;
            self.players[self.controlling].vel[match orientation {
                Horizontal => 0,
                Vertical => 1,
            }] = match self.pressing_state.map[orientation].solo_pressed() {
                Some(Positive) => SPEED,
                Some(Negative) => -SPEED,
                None => 0.,
            };
        }
        true
    }
}

impl DrivesMainLoop for GameState {
    fn render<B: Backend>(&mut self, _: &mut Renderer<B>) -> ProceedWith<(usize, &[DrawInfo])> {
        Ok((self.tex_id, &self.draw_infos))
    }

    fn update<B: Backend>(&mut self, renderer: &mut Renderer<B>) -> Proceed {
        self.update_player_positions();
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
