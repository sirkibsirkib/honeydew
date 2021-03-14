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
    const W: u16 = 32;
    const H: u16 = 32;
    const CELLS: u32 = Self::W as u32 * Self::H as u32;
    const BITS: u32 = Self::CELLS * 2;
    const CENTER: Coord = Coord([Self::W / 2, Self::H / 2]);
}

////////////
impl Coord {
    fn try_step(self, dir: Direction) -> Option<Self> {
        let Coord([x, y]) = self;
        Some(Self(match dir {
            Direction::Up if y > 0 => [x, y - 1],
            Direction::Down if y < RoomData::H - 1 => [x, y + 1],
            Direction::Left if x > 0 => [x - 1, y],
            Direction::Right if x < RoomData::W - 1 => [x + 1, y],
            _ => return None,
        }))
    }
}
impl Direction {
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
    fn get_cell(&self, Coord([x, y]): Coord) -> &RoomCellData {
        &self.data[y as usize][x as usize]
    }
    fn update_cells_with_step(&mut self, src: Coord, dest: Coord, dir: Direction) {
        use RoomCellData as Rcd;
        let coord = if dir.crossed_wall_at_src() { src } else { dest };
        let flags = Rcd::BLOCKED.with(if dir.horizontal() { Rcd::WALL_LE } else { Rcd::WALL_UP });
        self.get_mut_cell(coord).remove(flags);
    }
    fn new(rng: &mut Rng) -> Self {
        use RoomCellData as Rcd;
        let mut me = Self { data: [[Rcd::CLOSED; Self::W as usize]; Self::H as usize] };

        let mut at = Self::CENTER;
        let mut step_stack = Vec::<Direction>::with_capacity(64);
        loop {
            if let Some((dir, dest)) = me.dir_to_random_blocked_adjacent_to(rng, at) {
                me.update_cells_with_step(at, dest, dir);
                step_stack.push(dir);
            } else if let Some(dir) = step_stack.pop() {
                // backtrack
                todo!()
            } else {
                break;
            }
        }
        me
    }
    fn draw(&self) {
        use RoomCellData as Rcd;
        for row in self.data.iter() {
            // one row for vertical walls
            for cell in row.iter() {
                print!(".{}", if cell.superset_of(Rcd::WALL_UP) { '=' } else { ' ' });
            }
            println!();
            // one row for horizontal walls & blockages
            for cell in row.iter() {
                print!(
                    "{}{}",
                    if cell.superset_of(Rcd::WALL_LE) { '|' } else { ' ' },
                    if cell.superset_of(Rcd::BLOCKED) { '#' } else { ' ' }
                );
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
    fn shuffle<T>(&mut self, s: &mut [T]) {
        self.fastrand_rng.shuffle(s)
    }
    fn take_cache_bits(&mut self, bits: u8) -> u32 {
        assert!(bits <= 32);
        if bits >= self.cache_lsb_left {
            println!("REFILL");
            // refill cache
            self.cache = self.fastrand_rng.u32(..);
            self.cache_lsb_left = 32;
        }
        let ret = self.cache & !(!0 << bits);
        self.cache >>= bits;
        self.cache_lsb_left -= bits;
        ret
    }
    // fn gen_bool(&mut self) -> bool {
    //     self.take_cache_bits(1) > 0
    // }
    fn gen_direction(&mut self) -> Direction {
        match self.take_cache_bits(2) {
            0b00 => Direction::Up,
            0b01 => Direction::Down,
            0b10 => Direction::Left,
            0b11 => Direction::Right,
            _ => unreachable!(),
        }
    }
}
fn main() {
    let maybe_seed = Some(1);
    RoomData::new(&mut Rng::new(maybe_seed)).draw();
}
