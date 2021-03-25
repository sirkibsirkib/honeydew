pub mod rendering;
pub mod room;

use {
    crate::{bit_set::INDICES, prelude::*, rng::Rng},
    gfx_2020::{gfx_hal::Backend, winit::event::ElementState, *},
    room::{Coord, Room, CELL_SIZE},
};

pub const PLAYER_SIZE: DimMap<u16> =
    DimMap { arr: [CELL_SIZE.arr[0] / 4 * 3, CELL_SIZE.arr[1] / 4 * 3] };
pub const TELEPORTER_SIZE: DimMap<u16> =
    DimMap { arr: [CELL_SIZE.arr[0] / 3, CELL_SIZE.arr[1] / 3] };
pub const UP_WALL_SIZE: DimMap<u16> = DimMap { arr: [CELL_SIZE.arr[0], CELL_SIZE.arr[1] / 8] };
pub const MOV_SPEED: u16 = CELL_SIZE.arr[0] / 16;

// allows an upper bound for renderer's instance buffers
pub const MAX_TELEPORTERS: u32 = INDICES as u32 / 64;
pub const MAX_PLAYERS: u32 = 32;
pub const MAX_WALLS: u32 = INDICES as u32 * 2;
/////////////////////////////////

struct Rect {
    center: DimMap<WrapInt>,
    size: DimMap<u16>,
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
    pub teleporters: Vec<DimMap<WrapInt>>,
}

#[derive(Debug)]
pub struct Player {
    pub pos: DimMap<WrapInt>,
    pub vel: DimMap<Option<Sign>>,
}
#[derive(Default, Debug)]
pub struct PressingState {
    map: DimMap<AxisPressingState>,
}
#[derive(Copy, Clone, Debug)]
struct AxisPressingState {
    map: SignMap<ElementState>,
}
impl Default for AxisPressingState {
    fn default() -> Self {
        AxisPressingState { map: SignMap { arr: [ElementState::Released; 2] } }
    }
}

//////////////////////////

impl Player {
    fn try_dir_collide(&mut self, rect: &Rect, dir: Direction) -> bool {
        if !rect.contains(self.pos) {
            return false;
        }
        self.pos[dir.dim()] = rect.aligned_to_edge(-dir);
        true
    }
    fn try_collide(&mut self, rect: &Rect) -> bool {
        match [self.vel[X], self.vel[Y]] {
            [None, None] => false,
            [Some(sign), None] => self.try_dir_collide(rect, X.sign(sign)),
            [None, Some(sign)] => self.try_dir_collide(rect, Y.sign(sign)),
            [Some(x_sign), Some(y_sign)] => {
                if !rect.contains(self.pos) {
                    return false;
                }
                let new_x = rect.aligned_to_edge(X.sign(!x_sign));
                let new_y = rect.aligned_to_edge(Y.sign(!y_sign));
                println!("{:?} vs {:?}", [new_x, new_y], self.pos);
                if (new_x - self.pos[X]).distance_from_zero()
                    < (new_y - self.pos[Y]).distance_from_zero()
                {
                    println!("X is less drastic");
                    self.pos[X] = new_x;
                } else {
                    println!("Y is less drastic");
                    self.pos[Y] = new_y;
                }
                true
            }
        }
    }
}
impl Rect {
    fn aligned_to_edge(&self, dir: Direction) -> WrapInt {
        println!("{:?}", dir);
        let dim = dir.dim();
        self.center[dim] + dir.sign() * WrapInt::from(self.size[dim])
    }
    fn contains(&self, pt: DimMap<WrapInt>) -> bool {
        Dim::iter_domain()
            .all(|dim| (pt[dim] - self.center[dim]).distance_from_zero() < self.size[dim])
    }
}
impl World {
    pub fn random_free_space(&self, rng: &mut Rng) -> DimMap<WrapInt> {
        pub const MIN_DIST: DimMap<u16> =
            DimMap { arr: [CELL_SIZE.arr[0] * 2, CELL_SIZE.arr[1] * 2] };
        loop {
            let new = Coord::random(rng).into_vec2_center();
            let mut pos_iter =
                self.teleporters.iter().copied().chain(self.players.iter().map(|p| p.pos));
            if pos_iter.all(move |pos| {
                Dim::iter_domain()
                    .any(move |dim| (pos[dim] - new[dim]).distance_from_zero() >= MIN_DIST[dim])
            }) {
                return new;
            }
        }
    }
    fn move_and_collide(&mut self, net: &mut Net) {
        // update player positions wrt. movement
        for player in &mut self.players {
            // println!("{:?}", player.pos);
            for dim in Dim::iter_domain() {
                if let Some(sign) = player.vel[dim] {
                    player.pos[dim] += sign * WrapInt::from(MOV_SPEED);
                }
            }
        }

        // correct player positions wrt. player<->player collisions
        for [a, b] in iter_pairs_mut(&mut self.players) {
            a.try_collide(&Rect { center: b.pos, size: PLAYER_SIZE });
        }

        // teleporter <-> colliders

        if let Net::Server { rng, .. } = net {
            for i in 0..self.players.len() {
                let player_pos = self.players[i].pos;
                for j in 0..self.teleporters.len() {
                    let teleporter = self.teleporters[j];
                    let rect = Rect {
                        center: teleporter,
                        size: (PLAYER_SIZE + TELEPORTER_SIZE).map(|val| val / 2u16),
                    };
                    if rect.contains(player_pos) {
                        self.players[i].pos = self.random_free_space(rng);
                        self.teleporters[j] = self.random_free_space(rng);
                    }
                }
            }
        }

        for player in &mut self.players {
            println!("at {:?}", player.pos);

            // correct position wrt. player<->wall collisions
            for dim in Dim::iter_domain() {
                for check_at in Coord::check_for_collisions_at(dim, player.pos) {
                    if self.room.wall_sets[dim].contains(check_at.into()) {
                        let rect = Rect {
                            center: GameState::wall_pos(check_at, dim),
                            size: GameState::wall_min_dists(dim),
                        };
                        player.try_collide(&rect);
                    }
                }
            }
        }
    }
}
impl GameState {
    fn wall_min_dists(dim: Dim) -> DimMap<u16> {
        (Self::wall_size(dim) + PLAYER_SIZE).map(|val| val / 2u16)
    }
    pub fn wall_size(dim: Dim) -> DimMap<u16> {
        match dim {
            X => UP_WALL_SIZE,
            Y => UP_WALL_SIZE.transposed(),
        }
    }
    pub fn wall_pos(coord: Coord, dim: Dim) -> DimMap<WrapInt> {
        // e.g. Hdimzontal wall at Coord[0,0] has pos [0.5, 0.0]
        let mut pos = coord.into_vec2_corner();
        pos[dim] += CELL_SIZE[X] / 2;
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
        self.world.move_and_collide(&mut self.net);
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
