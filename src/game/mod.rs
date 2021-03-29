pub mod config;
pub mod net;
pub mod rendering;
pub mod room;

use {
    crate::{prelude::*, rng::Rng},
    config::Config,
    gfx_2020::{gfx_hal::Backend, winit::event::ElementState, *},
    net::Net,
    room::{Coord, Room, CELL_SIZE, TOT_CELL_COUNT},
};

pub const PLAYER_SIZE: DimMap<u16> =
    DimMap::new([CELL_SIZE.arr[0] / 9 * 5, CELL_SIZE.arr[1] / 9 * 5]);

pub const TELEPORTER_SIZE: DimMap<u16> = DimMap::new([CELL_SIZE.arr[0] / 3, CELL_SIZE.arr[1] / 3]);

pub const UP_WALL_SIZE: DimMap<u16> = DimMap::new([CELL_SIZE.arr[0], CELL_SIZE.arr[1] / 8]);
pub const LE_WALL_SIZE: DimMap<u16> = DimMap::new([CELL_SIZE.arr[0] / 8, CELL_SIZE.arr[1]]);

pub const MOV_SPEED: DimMap<u16> = DimMap::new([CELL_SIZE.arr[0] / 16, CELL_SIZE.arr[1] / 16]);

// allows an upper bound for renderer's instance buffers
pub const NUM_TELEPORTERS: u32 = TOT_CELL_COUNT as u32 / 64;
pub const NUM_PLAYERS: u32 = 3;
pub const MAX_WALLS: u32 = TOT_CELL_COUNT as u32 * 2;

/////////////////////////////////

pub type Pos = DimMap<WrapInt>;
pub type Vel = DimMap<Option<Sign>>;
pub type PlayerArr<T> = [T; NUM_PLAYERS as usize];

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PlayerColor {
    Black = 0,
    Blue = 1,
    Orange = 2,
}
#[derive(Debug, Copy, Clone)]
pub enum PlayerRelation {
    Predator,
    Prey,
    Identity,
}

struct Rect {
    center: Pos,
    size: DimMap<u16>,
}
pub struct GameState {
    pub world: World,
    // controlling
    pub controlling: PlayerColor,
    pub pressing_state: PressingState,
    // rendering
    pub tex_id: TexId,
    pub draw_infos: [DrawInfo; 4], // four replicas of all instances to pan the maze indefinitely
    // network
    pub net: Net,
}
pub struct World {
    pub room: Room,
    pub entities: Entities,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entities {
    pub players: PlayerArr<Player>,
    pub teleporters: [Pos; NUM_TELEPORTERS as usize],
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub pos: Pos,
    pub vel: Vel,
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
        AxisPressingState { map: SignMap::new([ElementState::Released; 2]) }
    }
}

//////////////////////////

impl Into<usize> for PlayerColor {
    fn into(self) -> usize {
        self as usize
    }
}
impl PlayerColor {
    pub fn peers(self) -> [Self; 2] {
        let prey = self.prey();
        [prey.prey(), prey]
    }
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        std::array::IntoIter::new([Self::Black, Self::Blue, Self::Orange])
    }
    pub fn relation_to(self, other: Self) -> PlayerRelation {
        if self == other {
            PlayerRelation::Identity
        } else if self == other.prey() {
            PlayerRelation::Prey
        } else {
            PlayerRelation::Predator
        }
    }
    pub fn related_by(self, rl: PlayerRelation) -> Self {
        match rl {
            PlayerRelation::Identity => self,
            PlayerRelation::Prey => self.prey(),
            PlayerRelation::Predator => self.prey().prey(),
        }
    }
    #[inline]
    pub fn prey(self) -> Self {
        match self {
            Self::Black => Self::Blue,
            Self::Blue => Self::Orange,
            Self::Orange => Self::Black,
        }
    }
}
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
                if (new_x - self.pos[X]).distance_from_zero()
                    < (new_y - self.pos[Y]).distance_from_zero()
                {
                    self.pos[X] = new_x;
                } else {
                    self.pos[Y] = new_y;
                }
                true
            }
        }
    }
}
impl Rect {
    fn aligned_to_edge(&self, dir: Direction) -> WrapInt {
        let dim = dir.dim();
        self.center[dim] + dir.sign() * WrapInt::from(self.size[dim])
    }
    fn contains(&self, pt: Pos) -> bool {
        Dim::iter_domain()
            .all(|dim| (pt[dim] - self.center[dim]).distance_from_zero() < self.size[dim])
    }
}
impl Entities {
    pub fn random(rng: &mut Rng) -> Self {
        let mut me = Self { players: Default::default(), teleporters: Default::default() };
        for i in 0..NUM_PLAYERS as usize {
            me.players[i].pos = me.random_free_space(rng);
        }
        for i in 0..NUM_TELEPORTERS as usize {
            me.teleporters[i] = me.random_free_space(rng);
        }
        me
    }
    pub fn random_free_space(&self, rng: &mut Rng) -> Pos {
        pub const MIN_DIST: DimMap<u16> = DimMap::new([CELL_SIZE.arr[0] * 2, CELL_SIZE.arr[1] * 2]);
        loop {
            let new = Coord::random(rng).center_pos();
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
}
impl World {
    fn move_and_collide(&mut self, net: &mut Net) {
        // update player positions wrt. movement
        for player in &mut self.entities.players {
            for dim in Dim::iter_domain() {
                if let Some(sign) = player.vel[dim] {
                    player.pos[dim] += sign * WrapInt::from(MOV_SPEED[dim]);
                }
            }
        }

        if let Some(rng) = net.server_rng() {
            // player<->player
            for predator in PlayerColor::iter_domain() {
                let prey = predator.prey();
                let rect = Rect { center: self.entities.players[prey].pos, size: PLAYER_SIZE };
                if rect.contains(self.entities.players[predator].pos) {
                    self.entities.players[prey].pos = self.entities.random_free_space(rng);
                }
            }

            // player<->teleporter
            for i in 0..self.entities.players.len() {
                let player_pos = self.entities.players[i].pos;
                for j in 0..self.entities.teleporters.len() {
                    let teleporter = self.entities.teleporters[j];
                    let rect = Rect {
                        center: teleporter,
                        size: (PLAYER_SIZE + TELEPORTER_SIZE).map(|val| val / 2u16),
                    };
                    if rect.contains(player_pos) {
                        self.entities.players[i].pos = self.entities.random_free_space(rng);
                        self.entities.teleporters[j] = self.entities.random_free_space(rng);
                    }
                }
            }
        }

        for player in &mut self.entities.players {
            // correct position wrt. player<->wall collisions
            for dim in Dim::iter_domain() {
                for check_at in Room::wall_cells_to_check_at(player.pos, dim) {
                    if self.room.wall_sets[dim].contains(check_at.bit_index()) {
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
    pub fn new<B: Backend>(renderer: &mut Renderer<B>, config: &Config) -> Self {
        let tex_id = renderer.load_texture({
            let image_bytes = include_bytes!("faces.png");
            &gfx_2020::load_texture_from_bytes(image_bytes).expect("Failed to decode png!")
        });
        let (net, world, controlling) = Net::new(config);
        let state = GameState {
            net,
            world,
            pressing_state: Default::default(),
            tex_id,
            draw_infos: GameState::init_draw_infos(),
            controlling,
        };
        state.init_vertex_buffers(renderer);
        state
    }
    fn wall_min_dists(dim: Dim) -> DimMap<u16> {
        (Self::wall_size(dim) + PLAYER_SIZE).map(|val| val / 2u16)
    }
    pub fn wall_size(dim: Dim) -> DimMap<u16> {
        match dim {
            X => UP_WALL_SIZE,
            Y => LE_WALL_SIZE,
        }
    }
    pub fn wall_pos(coord: Coord, dim: Dim) -> Pos {
        // e.g. X dim wall at Coord[0,0] has pos [0.5, 0.0]
        let mut pos = coord.corner_pos();
        pos[dim] += CELL_SIZE[dim] / 2;
        pos
    }
    fn update_move_key(&mut self, dir: Direction, state: ElementState) {
        let dim = dir.dim();
        self.pressing_state.map[dim].map[dir.sign()] = state;
        self.world.entities.players[self.controlling].vel[dim] =
            self.pressing_state.map[dim].solo_pressed();
    }
}

impl DrivesMainLoop for GameState {
    fn render<B: Backend>(
        &mut self,
        _: &mut Renderer<B>,
    ) -> ProceedWith<(usize, ClearColor, &[DrawInfo])> {
        Ok(self.get_draw_args())
    }

    fn update<B: Backend>(&mut self, renderer: &mut Renderer<B>) -> Proceed {
        self.world.move_and_collide(&mut self.net);
        self.net.update(self.controlling, &mut self.world.entities);
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
impl<T> Index<PlayerColor> for PlayerArr<T> {
    type Output = T;
    fn index(&self, idx: PlayerColor) -> &T {
        &self[Into::<usize>::into(idx)]
    }
}
impl<T> IndexMut<PlayerColor> for PlayerArr<T> {
    fn index_mut(&mut self, idx: PlayerColor) -> &mut T {
        &mut self[Into::<usize>::into(idx)]
    }
}
