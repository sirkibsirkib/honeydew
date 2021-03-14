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
    fn coord_of_random_free_adjacent_to(&self, coord: Coord) -> Option<Coord> {}
    fn get_mut_cell(&mut self, Coord([x, y]): Coord) -> &mut RoomCellData {
        &mut self.data[y as usize][x as usize]
    }
    fn new(rng: &mut Rng) -> Self {
        use RoomCellData as Rcd;
        let mut me = Self { data: [[Rcd::CLOSED; Self::W as usize]; Self::H as usize] };

        let mut at = Self::CENTER;
        let mut step_stack = Vec::<Direction>::with_capacity(64);
        loop {
            // step stuff
            if step_stack.is_empty() {
                break;
            }
        }

        // let mut carvers_at: Vec<Coord> = vec![Self::CENTER];
        // me.get_mut_cell(Self::CENTER).remove(Rcd::BLOCKED);
        // for step in 0..64 {
        //     if step % 8 == 0 {
        //         // duplicate the most recent carver
        //         carvers_at.push(*carvers_at.iter().last().unwrap());
        //     }
        //     // advance all carvers randomly
        //     for carver_at in carvers_at.iter_mut() {
        //         let dir = rng.gen_direction();
        //         if let Some(dest) = carver_at.try_step(dir) {
        //             // ok the carver can move that way.
        //             {
        //                 // remove the wall between src and dest cell
        //                 let wall_coord = if dir.crossed_wall_at_src() { *carver_at } else { dest };
        //                 let wall_flag = if dir.horizontal() { Rcd::WALL_LE } else { Rcd::WALL_UP };
        //                 me.get_mut_cell(wall_coord).remove(wall_flag);
        //             }
        //             // remove the blockage at dest cell
        //             me.get_mut_cell(dest).remove(Rcd::BLOCKED);
        //             // update the carver's position
        //             *carver_at = dest;
        //         }
        //     }
        // }
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
