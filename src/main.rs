#[derive(Eq, PartialEq, Copy, Clone)]
struct RoomCellData(u8);

struct Rng {
    fastrand_rng: fastrand::Rng,
    cache: u32,
    cache_lsb_left: u8,
}

#[derive(Copy, Clone, Debug)]
struct Coord([u16; 2]);

struct RoomData {
    data: [[RoomCellData; Self::W as usize]; Self::H as usize],
}
#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
////////////////////////////

impl RoomCellData {
    const BLOCKED: Self = Self(0b00000001);
    const WALL_UP: Self = Self(0b00000010);
    const WALL_LE: Self = Self(0b00000100);
    const TELEPOR: Self = Self(0b00001000);
    ////////
    const OPEN: Self = Self(0);
    const CLOSED: Self = Self::BLOCKED.with(Self::WALL_UP).with(Self::WALL_LE);
    const fn with(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
    const fn without(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
    const fn subset_of(self, rhs: Self) -> bool {
        self.without(rhs).0 == 0
    }
    const fn superset_of(self, rhs: Self) -> bool {
        rhs.subset_of(self)
    }
    fn add(&mut self, rhs: Self) {
        *self = self.with(rhs);
    }
    fn remove(&mut self, rhs: Self) {
        *self = self.without(rhs);
    }
}
impl RoomData {
    const W: u16 = 64;
    const H: u16 = 64;
    const CELLS: u32 = Self::W as u32 * Self::H as u32;
    const CENTER: Coord = Coord([Self::W / 2, Self::H / 2]);
}

////////////
impl Coord {
    fn try_step(self, dir: Direction) -> Option<Self> {
        let Coord([x, y]) = self;
        let xy = match dir {
            Direction::Up if y > 0 => [x, y - 1],
            Direction::Left if x > 0 => [x - 1, y],
            Direction::Down if y < RoomData::H - 2 => [x, y + 1],
            Direction::Right if x < RoomData::W - 2 => [x + 1, y],
            _ => return None,
        };
        Some(Self(xy))
    }
}
impl Direction {
    fn invert(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
    fn crossed_wall_at_src(self) -> bool {
        match self {
            Self::Down | Self::Right => false,
            Self::Up | Self::Left => true,
        }
    }
    fn horizontal(self) -> bool {
        match self {
            Self::Up | Self::Down => false,
            Self::Left | Self::Right => true,
        }
    }
}

impl RoomData {
    fn coord_iter() -> impl Iterator<Item = Coord> {
        (0..Self::H).flat_map(|y| (0..Self::W).map(move |x| Coord([x, y])))
    }
    fn iter_cells(&self) -> impl Iterator<Item = (Coord, RoomCellData)> + '_ {
        Self::coord_iter().map(move |coord| (coord, self.get_cell(coord)))
    }
    fn iter_walls(&self) -> impl Iterator<Item = (Coord, bool)> + '_ {
        self.iter_cells().flat_map(|(coord, cell)| {
            let some_coord = |up, some| if some { Some((coord, up)) } else { None };
            some_coord(true, cell.subset_of(RoomCellData::WALL_UP))
                .into_iter()
                .chain(some_coord(false, cell.subset_of(RoomCellData::WALL_LE)))
        })
    }
    fn dir_to_random_blocked_adjacent_to(
        &self,
        rng: &mut Rng,
        src: Coord,
    ) -> Option<(Direction, Coord)> {
        let mut dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        rng.shuffle(&mut dirs);
        let steps_to_blocked = move |dir: Direction| {
            if let Some(dest) = src.try_step(dir) {
                if self.get_cell(dest).superset_of(RoomCellData::BLOCKED) {
                    return Some((dir, dest));
                }
            }
            None
        };
        dirs.iter().copied().filter_map(steps_to_blocked).next()
    }
    fn get_mut_cell(&mut self, Coord([x, y]): Coord) -> &mut RoomCellData {
        &mut self.data[y as usize][x as usize]
    }
    fn get_cell(&self, Coord([x, y]): Coord) -> RoomCellData {
        self.data[y as usize][x as usize]
    }
    fn update_cells_with_step(&mut self, [src, dest]: [Coord; 2], dir: Direction) {
        use RoomCellData as Rcd;
        self.get_mut_cell(dest).remove(Rcd::BLOCKED);
        let coord = if dir.crossed_wall_at_src() { src } else { dest };
        let flags = if dir.horizontal() { Rcd::WALL_LE } else { Rcd::WALL_UP };
        self.get_mut_cell(coord).remove(flags);
    }
    fn new(maybe_seed: Option<u64>) -> Self {
        use RoomCellData as Rcd;
        let rng = &mut Rng::new(maybe_seed);
        let mut me = Self { data: [[Rcd::CLOSED; Self::W as usize]; Self::H as usize] };

        let mut at = Self::CENTER;
        let mut step_stack = Vec::<Direction>::with_capacity(1 << 12);
        me.get_mut_cell(at).remove(Rcd::BLOCKED);
        loop {
            if let Some((dir, dest)) = me.dir_to_random_blocked_adjacent_to(rng, at) {
                me.update_cells_with_step([at, dest], dir);
                at = dest;
                step_stack.push(dir);
            } else if let Some(dir) = step_stack.pop() {
                // backtrack
                at = at.try_step(dir.invert()).unwrap();
            } else {
                break;
            }
        }
        for _ in 0..(RoomData::CELLS / 8) {
            let flag = if rng.gen_bool() { Rcd::WALL_LE } else { Rcd::WALL_UP };
            let coord = Coord([
                rng.fastrand_rng.u16(1..RoomData::W - 1), // x
                rng.fastrand_rng.u16(1..RoomData::H - 1),
            ]);
            me.get_mut_cell(coord).remove(flag);
        }
        me
    }
    fn draw(&self) {
        use RoomCellData as Rcd;
        for row in self.data.iter() {
            // one row for vertical walls
            for cell in row.iter() {
                let up_char = if cell.superset_of(Rcd::WALL_UP) { '―' } else { ' ' };
                print!("·{}{}", up_char, up_char);
            }
            println!();
            // one row for horizontal walls & blockages
            for cell in row.iter() {
                let left_char = if cell.superset_of(Rcd::WALL_LE) { '|' } else { ' ' };
                let blocked_char = if cell.superset_of(Rcd::BLOCKED) { '#' } else { ' ' };
                print!("{}{}{}", left_char, blocked_char, blocked_char);
            }
            println!();
        }
    }
}

impl Rng {
    fn new(maybe_seed: Option<u64>) -> Self {
        let fastrand_rng = if let Some(seed) = maybe_seed {
            fastrand::Rng::with_seed(seed)
        } else {
            fastrand::Rng::new()
        };
        Self { fastrand_rng, cache: 0, cache_lsb_left: 0 }
    }
    fn gen_bits(&mut self, bits: u8) -> u32 {
        assert!(bits <= 32);
        if self.cache_lsb_left < bits {
            self.cache = self.fastrand_rng.u32(..);
            self.cache_lsb_left = 32;
        }
        let ret = self.cache & !(!0 << bits);
        self.cache_lsb_left -= bits;
        self.cache >>= bits;
        ret
    }
    fn gen_bool(&mut self) -> bool {
        self.gen_bits(1) != 0
    }
    fn shuffle<T>(&mut self, s: &mut [T]) {
        self.fastrand_rng.shuffle(s)
    }
}

use {
    gfx_2020::{gfx_hal::Backend, vert_coord_consts::UNIT_QUAD, *},
    gfx_backend_vulkan::Backend as VulkanBackend,
};

struct GameState {
    room_data: RoomData,
    tex_id: TexId,
    draw_infos: [DrawInfo; 1],
}

fn game_state_init_fn<B: Backend>(renderer: &mut Renderer<B>) -> GameState {
    let texture = gfx_2020::load_texture_from_path("./src/data/faces.png").unwrap();
    let tex_id = renderer.load_texture(&texture);
    println!("{:?}", tex_id);
    let room_data = RoomData::new(Some(0));

    renderer.write_vertex_buffer(0, UNIT_QUAD.iter().copied());
    let mut instance_count: u32 = 0;
    renderer.write_vertex_buffer(
        0,
        room_data.iter_walls().map(|(Coord([x, y]), wall_up)| {
            let [mut tx, mut ty] = [x as f32, y as f32];
            *if wall_up { &mut ty } else { &mut tx } -= 0.5;
            instance_count += 1;
            Mat4::from_translation(Vec3::new(tx, ty, 0.))
                * Mat4::from_rotation_z(if wall_up { 0. } else { std::f32::consts::PI / 2. })
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
        view_transform: Mat4::from_scale(Vec3::new(0.1, 0.1, 1.))
            * Mat4::from_translation(Vec3::new(
                RoomData::W as f32 * -0.5,
                RoomData::H as f32 * -0.5,
                0.,
            )),
        vertex_range: 0..UNIT_QUAD.len() as u32,
    }];
    GameState { room_data, tex_id, draw_infos }
}

fn heap_leak<T>(t: T) -> &'static mut T {
    Box::leak(Box::new(t))
}

fn main() {
    let config = RendererConfig {
        max_buffer_args: MaxBufferArgs {
            max_tri_verts: UNIT_QUAD.len() as u32,
            max_instances: 2048,
        },
        ..Default::default()
    };
    main_loop::<VulkanBackend, _, _>(&config, |x| Ok(heap_leak(game_state_init_fn(x))))
}
impl DrivesMainLoop for GameState {
    fn render<B>(&mut self, _: &mut Renderer<B>) -> ProceedWith<(usize, &[DrawInfo])>
    where
        B: Backend,
    {
        Ok((self.tex_id, &self.draw_infos))
    }
}
