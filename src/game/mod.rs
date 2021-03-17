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
/////////////////////////////////

pub struct GameState {
    pub room: Room,
    pub controlling: usize,
    pub players: [Player; 1],
    pub pressing_state: PressingState,
    pub tex_id: TexId,
    pub draw_infos: [DrawInfo; 1],
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
                Sign::Negative => ElementState::Released,
                Sign::Positive => ElementState::Released,
            },
        }
    }
}
impl GameState {
    pub(crate) fn calc_view_transform(pos: Vec3) -> Mat4 {
        Mat4::from_scale(Vec3::new(0.2, 0.2, 1.0)) * Mat4::from_translation(-pos)
    }
    fn pressing_state_update(&mut self, vkc: VirtualKeyCode, state: ElementState) -> bool {
        use {Orientation::*, Sign::*, VirtualKeyCode as Vkc};
        const SPEED: f32 = 0.03;

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

    fn update<B: Backend>(&mut self, _: &mut Renderer<B>) -> Proceed {
        for player in self.players.iter_mut() {
            player.pos += player.vel;
        }
        self.draw_infos[0].view_transform =
            Self::calc_view_transform(self.players[self.controlling].pos);
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
        use {ElementState::*, Sign::*};
        match [self.map[Negative], self.map[Positive]] {
            [Pressed, Pressed] | [Released, Released] => None,
            [Pressed, Released] => Some(Negative),
            [Released, Pressed] => Some(Positive),
        }
    }
}
