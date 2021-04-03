pub mod ai;
pub mod config;
pub mod net;
pub mod rendering;
pub mod room;

use {
    crate::{prelude::*, rng::Rng},
    ai::{Ai, AiExt, SinkAi},
    config::{Config, InputConfig},
    gfx_2020::{gfx_hal::Backend, winit::event::ElementState, *},
    net::{Client, Server},
    room::{Coord, Room, CELL_SIZE, TOT_CELL_COUNT},
};

pub const MOVE_SPEED: u16 = 16;
pub const MOVE_SIZE: Size = CELL_SIZE.scalar_div(MOVE_SPEED);
pub const MOVE_SIZE_DIAG: Size = MOVE_SIZE.scalar_mul(5).scalar_div(7);

pub const NUM_DRAW_INFOS: usize = 4;

pub const PLAYER_SIZE: Size = CELL_SIZE.scalar_div(9).scalar_mul(4);
pub const TELEPORTER_SIZE: Size = CELL_SIZE.scalar_div(2);
pub const WALL_SIZE: DimMap<Size> = DimMap::new([
    Size::new([CELL_SIZE.arr[0], CELL_SIZE.arr[1] / 7]),
    Size::new([CELL_SIZE.arr[0] / 7, CELL_SIZE.arr[1]]),
]);
pub const ZERO_POS: Pos = Pos::new([WrapInt::ZERO; 2]);

// allows an upper bound for renderer's instance buffers
pub const NUM_TELEPORTERS: u32 = TOT_CELL_COUNT as u32 / 64;
pub const NUM_PLAYERS: u32 = 3;
pub const MAX_WALLS: u32 = TOT_CELL_COUNT as u32 * 2;
pub const NUM_MY_DOORS: u32 = MAX_WALLS as u32 / 64;

/////////////////////////////////

pub type Pos = DimMap<WrapInt>;
pub type Size = DimMap<u16>;
pub type Vel = DimMap<Option<Sign>>;
pub type PlayerArr<T> = [T; NUM_PLAYERS as usize];

#[derive(Eq, PartialEq, Copy, Clone, Default)]
pub struct MyDoorIndexSet {
    bits: u16,
}
#[derive(Copy, Clone)]
pub struct MyDoor {
    moving_through: Option<Sign>,
    coord: Coord,
    dim: Dim,
}

pub enum Net {
    Server { server: Server, ais: PlayerArr<Option<SinkAi>> },
    Client(Client),
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PlayerColor {
    Black = 0,
    Blue = 1,
    Orange = 2,
}
struct Rect {
    center: Pos,
    size: Size,
}
pub struct GameState {
    pub world: World,
    pub my_doors: [MyDoor; NUM_MY_DOORS as usize],
    // pub currently_inside_doors: MyDoorIndexSet,
    // controlling
    pub controlling: PlayerColor,
    pub pressing_state: PressingState,
    pub input_config: InputConfig,
    // rendering
    pub tex_id: TexId,
    pub draw_infos: [DrawInfo; NUM_DRAW_INFOS], // four replicas of all instances to pan the maze indefinitely
    pub net: Net,
    pub local_rng: Rng,
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
struct PrettyPos {
    pos: Pos,
}

//////////////////////////
impl std::fmt::Debug for PrettyPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let e_iter = Dim::iter_domain().map(move |dim| {
            let val: i16 = self.pos[dim].into();
            let val = val as f32 / CELL_SIZE[dim] as f32;
            (dim, val)
        });
        f.debug_map().entries(e_iter).finish()
    }
}
impl MyDoorIndexSet {
    pub fn inserted(mut self, idx: usize) -> Self {
        self.bits |= 1 << idx;
        self
    }
    pub fn insert(&mut self, idx: usize) {
        *self = self.inserted(idx)
    }
    pub fn contains(self, idx: usize) -> bool {
        self.inserted(idx) == self
    }
}

impl Into<usize> for PlayerColor {
    fn into(self) -> usize {
        self as usize
    }
}
impl PlayerColor {
    pub fn predator_prey(self) -> [Self; 2] {
        [self.predator(), self.prey()]
    }
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        std::array::IntoIter::new([Self::Black, Self::Blue, Self::Orange])
    }
    #[inline]
    pub fn predator(self) -> Self {
        self.prey().prey()
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
    // only call IF you know you're colliding
    fn snap_pos_wrt_vel(&mut self, rect: &Rect) {
        fn snap_pos_dim_to(player: &mut Player, rect: &Rect, dir: Direction) {
            player.pos[dir.dim()] = rect.aligned_to_edge(-dir);
        }
        match [self.vel[X], self.vel[Y]] {
            [None, None] => {}
            [Some(sign), None] => snap_pos_dim_to(self, rect, X.sign(sign)),
            [None, Some(sign)] => snap_pos_dim_to(self, rect, Y.sign(sign)),
            [Some(x_sign), Some(y_sign)] => {
                let new_x = rect.aligned_to_edge(X.sign(!x_sign));
                let new_y = rect.aligned_to_edge(Y.sign(!y_sign));
                if (new_x - self.pos[X]).distance_from_zero()
                    < (new_y - self.pos[Y]).distance_from_zero()
                {
                    self.pos[X] = new_x;
                } else {
                    self.pos[Y] = new_y;
                }
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
        pub const MIN_DIST: Size = CELL_SIZE.scalar_mul(2);
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

impl Room {
    fn random_new_my_doors(&self, rng: &mut Rng) -> [MyDoor; NUM_MY_DOORS as usize] {
        let mut my_doors = [MyDoor { coord: Default::default(), dim: X, moving_through: None };
            NUM_MY_DOORS as usize];
        for i in 0..NUM_MY_DOORS as usize {
            my_doors[i] = self.random_new_my_door(rng, &my_doors);
        }
        my_doors
    }
    fn random_new_my_door(
        &self,
        rng: &mut Rng,
        my_doors: &[MyDoor; NUM_MY_DOORS as usize],
    ) -> MyDoor {
        loop {
            let coord = Coord::random(rng);
            // 1. check if its far away enough
            let too_close = my_doors.iter().any(|other| {
                (coord.corner_pos() - other.coord.corner_pos()).distances_from_zero()
                    < CELL_SIZE.scalar_mul(2)
            });
            if !too_close {
                let dim_iter = ArrIter::new(if rng.gen_bool() { [X, Y] } else { [Y, X] });
                if let Some(dim) =
                    dim_iter.filter(|&dim| self.wall_sets[dim].contains(coord.bit_index())).next()
                {
                    return MyDoor { coord, dim, moving_through: None };
                }
            }
        }
    }
}
impl GameState {
    fn update_net_and_ais(&mut self) {
        match &mut self.net {
            Net::Server { server, ais } => {
                for col in PlayerColor::iter_domain() {
                    if let Some(ai) = &mut ais[col] {
                        self.world.entities.players[col].vel =
                            ai.update(&self.world, &mut self.local_rng);
                    }
                }

                let new_client_callback = move |color: PlayerColor, entities: &mut Entities| {
                    ais[color] = None;
                    entities.players[color].vel = Vel::default();
                };
                server.update(self.controlling, &mut self.world.entities, new_client_callback)
            }
            Net::Client(client) => client.update(self.controlling, &mut self.world.entities),
        }
    }
    fn move_and_collide(&mut self) {
        // TODO if I am inside a door, mutate vel s.t. I continue going through
        // player movement
        for col in PlayerColor::iter_domain() {
            let player = &mut self.world.entities.players[col];
            let mut effective_vel = player.vel;
            if col == self.controlling {
                // override the player's input vel to continue moving them through the door
                for my_door in self.my_doors.iter() {
                    if let Some(sign) = my_door.moving_through {
                        effective_vel[!my_door.dim] = Some(sign);
                    }
                }
            }
            let move_size = if effective_vel[X].is_some() && effective_vel[Y].is_some() {
                MOVE_SIZE_DIAG
            } else {
                MOVE_SIZE
            };
            for dim in Dim::iter_domain() {
                if let Some(sign) = effective_vel[dim] {
                    player.pos[dim] += sign * WrapInt::from(move_size[dim]);
                }
            }
        }

        if let Net::Server { ais, .. } = &mut self.net {
            // player -> player collision
            for predator in PlayerColor::iter_domain() {
                let prey = predator.prey();
                let rect =
                    Rect { center: self.world.entities.players[prey].pos, size: PLAYER_SIZE };
                if rect.contains(self.world.entities.players[predator].pos) {
                    self.world.entities.players[prey].pos =
                        self.world.entities.random_free_space(&mut self.local_rng);
                    if let Some(ai) = &mut ais[prey] {
                        ai.i_was_moved(&self.world);
                    }
                }
            }

            // player -> teleporter collision
            for i in 0..self.world.entities.players.len() {
                let player_pos = self.world.entities.players[i].pos;
                for j in 0..self.world.entities.teleporters.len() {
                    let teleporter = self.world.entities.teleporters[j];
                    let rect = Rect {
                        center: teleporter,
                        size: (PLAYER_SIZE + TELEPORTER_SIZE).map(|val| val / 2u16),
                    };
                    if rect.contains(player_pos) {
                        self.world.entities.players[i].pos =
                            self.world.entities.random_free_space(&mut self.local_rng);
                        self.world.entities.teleporters[j] =
                            self.world.entities.random_free_space(&mut self.local_rng);
                        if let Some(ai) = &mut ais[i] {
                            ai.i_was_moved(&self.world);
                        }
                    }
                }
            }
        }

        // player -> wall collision
        let player = &mut self.world.entities.players[self.controlling];
        let mut moving_through_doors = MyDoorIndexSet::default(); // still building...
        for dim in Dim::iter_domain() {
            for coord in Room::wall_cells_to_check_at(player.pos, dim) {
                let wall_here = self.world.room.wall_sets[dim].contains(coord.bit_index());
                if !wall_here {
                    // no wall -> no door. nothing to do here.
                    continue;
                }
                let rect = Rect {
                    center: GameState::wall_pos(coord, dim),
                    size: (WALL_SIZE[dim] + PLAYER_SIZE).map(|val| val / 2u16),
                };
                let colliding = rect.contains(player.pos);
                if !colliding {
                    // no wall collision -> no door collision.
                    continue;
                }
                let my_door_here_idx = self
                    .my_doors
                    .iter()
                    .enumerate()
                    .filter(|(_, my_door)| my_door.dim == dim && my_door.coord == coord)
                    .map(|(index, _)| index)
                    .next();

                if let Some(i) = my_door_here_idx {
                    let my_door = &mut self.my_doors[i];
                    if my_door.moving_through.is_none() {
                        // moving through door START
                        my_door.moving_through = Some(if player.pos[!dim] < rect.center[!dim] {
                            Positive
                        } else {
                            Negative
                        });
                    }
                    moving_through_doors.insert(i);
                } else {
                    player.snap_pos_wrt_vel(&rect);
                }
            }
        }
        for i in (0..NUM_MY_DOORS as usize).filter(|&i| !moving_through_doors.contains(i)) {
            if self.my_doors[i].moving_through.take().is_some() {
                // moving through door END
                self.my_doors[i] =
                    self.world.room.random_new_my_door(&mut self.local_rng, &self.my_doors);
            }
        }
    }
    pub fn new<B: Backend>(renderer: &mut Renderer<B>, config: &Config) -> Self {
        let tex_id = renderer.load_texture({
            let image_bytes = include_bytes!("spritesheet.png");
            &gfx_2020::load_texture_from_bytes(image_bytes).expect("Failed to decode png!")
        });
        let mut local_rng = Rng::new_seeded(Rng::random_seed());
        let (net, world, controlling) = if config.server_mode {
            let (server, world, controlling) = Server::new(&config.if_server);
            let mut ais = PlayerArr::<Option<_>>::default();
            for &col in config.if_server.ai_enabled.iter() {
                if ais[col].is_none() && col != controlling {
                    ais[col] = Some(SinkAi::new(col, &world, &mut local_rng));
                }
            }
            let net = Net::Server { server, ais };
            (net, world, controlling)
        } else {
            let (client, world, controlling) = Client::new(&config.if_client);
            let net = Net::Client(client);
            (net, world, controlling)
        };
        let mut state = GameState {
            my_doors: world.room.random_new_my_doors(&mut local_rng),
            net,
            world,
            pressing_state: Default::default(),
            tex_id,
            draw_infos: GameState::init_draw_infos(),
            controlling,
            local_rng,
            input_config: config.input.clone(),
        };
        state.init_vertex_buffers(renderer);
        state
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
        self.move_and_collide();
        self.update_net_and_ais();
        self.update_vertex_buffers(renderer);
        self.update_view_transforms();
        Ok(())
    }

    fn handle_event<B: Backend>(
        &mut self,
        _renderer: &mut Renderer<B>,
        event: winit::event::Event<()>,
    ) -> Proceed {
        use winit::event::{Event as Ev, KeyboardInput as Ki, WindowEvent as We};
        match event {
            Ev::WindowEvent { event: We::CloseRequested, .. } => return Err(HaltLoop),
            Ev::WindowEvent { event: We::KeyboardInput { input, .. }, .. } => {
                // ok
                match input {
                    Ki { virtual_keycode: Some(vkc), state, .. } => match vkc {
                        x if x == self.input_config.exit => return Err(HaltLoop),
                        x if x == self.input_config.up => self.update_move_key(Up, state),
                        x if x == self.input_config.down => self.update_move_key(Down, state),
                        x if x == self.input_config.left => self.update_move_key(Left, state),
                        x if x == self.input_config.right => self.update_move_key(Right, state),
                        _ => {}
                    },
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

impl Size {
    const fn scalar_mul(mut self, rhs: u16) -> Self {
        self.arr[0] *= rhs;
        self.arr[1] *= rhs;
        self
    }
    const fn scalar_div(mut self, rhs: u16) -> Self {
        self.arr[0] /= rhs;
        self.arr[1] /= rhs;
        self
    }
}
impl Pos {
    fn distances_from_zero(self) -> Size {
        Size::new_xy_with(move |dim| self[dim].distance_from_zero())
    }
}
