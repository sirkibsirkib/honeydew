use {
    crate::{
        room::{Coord, Orientation, Room},
        GameState,
    },
    gfx_2020::{
        gfx_hal::{window::Extent2D, Backend},
        vert_coord_consts::UNIT_QUAD,
        *,
    },
};

pub fn render_config() -> RendererConfig<'static> {
    RendererConfig {
        init: RendererInitConfig {
            window_dims: Extent2D { width: 900, height: 900 },
            ..Default::default()
        },
        max_buffer_args: MaxBufferArgs {
            max_tri_verts: UNIT_QUAD.len() as u32,
            max_instances: 2048,
        },
        ..Default::default()
    }
}

pub(crate) fn game_state_init_fn<B: Backend>(
    renderer: &mut Renderer<B>,
) -> ProceedWith<&'static mut GameState> {
    let texture = gfx_2020::load_texture_from_path("./src/data/faces.png").unwrap();
    let tex_id = renderer.load_texture(&texture);
    let room = Room::new(Some(0));
    room.ascii_print();

    renderer.write_vertex_buffer(0, UNIT_QUAD.iter().copied());
    let mut instance_count: u32 = 0;
    renderer.write_vertex_buffer(
        0,
        room.iter_walls().map(|(Coord([x, y]), orientation)| {
            let [mut tx, mut ty] = [x as f32, y as f32];
            let (coord, rot) = match orientation {
                Orientation::Vertical => (&mut tx, 0.),
                Orientation::Horizontal => (&mut ty, std::f32::consts::PI / 2.),
            };
            *coord -= 0.5;
            instance_count += 1;
            Mat4::from_translation(Vec3::new(tx, ty, 0.))
                * Mat4::from_rotation_z(rot)
                * Mat4::from_scale(Vec3::new(1.0, 0.16, 1.0))
        }),
    );

    renderer.write_vertex_buffer(
        0,
        std::iter::repeat(TexScissor {
            top_left: [0., 0.],
            size: [6.0f32.recip(), 3.0f32.recip()],
        })
        .take(instance_count as usize),
    );
    let draw_infos = [DrawInfo {
        instance_range: 0..instance_count,
        view_transform: Mat4::from_scale(Vec3::new(0.15, 0.15, 1.))
            * Mat4::from_translation(Vec3::new(Room::W as f32 * -0.5, Room::H as f32 * -0.5, 0.)),
        vertex_range: 0..UNIT_QUAD.len() as u32,
    }];
    Ok(heap_leak(GameState { room, tex_id, draw_infos }))
}

pub fn heap_leak<T>(t: T) -> &'static mut T {
    Box::leak(Box::new(t))
}

impl DrivesMainLoop for GameState {
    fn render<B>(&mut self, _: &mut Renderer<B>) -> ProceedWith<(usize, &[DrawInfo])>
    where
        B: Backend,
    {
        Ok((self.tex_id, &self.draw_infos))
    }

    fn handle_event<B>(
        &mut self,
        _renderer: &mut Renderer<B>,
        event: winit::event::Event<()>,
    ) -> Proceed
    where
        B: Backend,
    {
        use winit::event::{
            Event as Ev, KeyboardInput as Ki, VirtualKeyCode as Vkc, WindowEvent as We,
        };
        let translate = |t: &mut Mat4, [x, y]: [f32; 2]| {
            *t = t.mul_mat4(&Mat4::from_translation(Vec3::new(x, y, 0.)))
        };
        match event {
            Ev::WindowEvent { event: We::CloseRequested, .. } => return Err(HaltLoop),
            Ev::WindowEvent { event: We::KeyboardInput { input, .. }, .. } => {
                // ok
                match input {
                    Ki { virtual_keycode: Some(Vkc::Escape), .. } => return Err(HaltLoop),
                    Ki { virtual_keycode: Some(Vkc::A), .. } => {
                        translate(&mut self.draw_infos[0].view_transform, [0.1, 0.0])
                    }
                    Ki { virtual_keycode: Some(Vkc::W), .. } => {
                        translate(&mut self.draw_infos[0].view_transform, [0.0, 0.1])
                    }
                    Ki { virtual_keycode: Some(Vkc::S), .. } => {
                        translate(&mut self.draw_infos[0].view_transform, [0.0, -0.1])
                    }
                    Ki { virtual_keycode: Some(Vkc::D), .. } => {
                        translate(&mut self.draw_infos[0].view_transform, [-0.1, 0.0])
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}
