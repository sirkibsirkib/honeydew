// # Imports and constants
use crate::rng::Rng;

///////////////////////////////////////////////
// # Data types

#[derive(Copy, Clone, Debug)]
pub struct Coord(pub [u16; 2]);

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Cell {
    flags: u8,
}

pub struct Room {
    data: [[Cell; Self::W as usize]; Self::H as usize],
}
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

///////////////////////////////////////////////
// # associated consts
impl Cell {
    pub const OPEN: Self = Self { flags: 0b00000000 };
    pub const BLOCKED: Self = Self { flags: 0b00000001 };
    pub const WALL_UP: Self = Self { flags: 0b00000010 };
    pub const WALL_LE: Self = Self { flags: 0b00000100 };
    pub const TELEPOR: Self = Self { flags: 0b00001000 };
    ////////
    pub const CLOSED: Self = Self::BLOCKED.with(Self::WALL_UP).with(Self::WALL_LE);
}

///////////////////////////////////////////////
// # methods and functions

impl Cell {
    pub const fn with(self, rhs: Self) -> Self {
        Self { flags: self.flags | rhs.flags }
    }
    pub const fn without(self, rhs: Self) -> Self {
        Self { flags: self.flags & !rhs.flags }
    }
    pub const fn subset_of(self, rhs: Self) -> bool {
        self.without(rhs).flags == 0
    }
    pub const fn superset_of(self, rhs: Self) -> bool {
        rhs.subset_of(self)
    }
    pub fn add(&mut self, rhs: Self) {
        *self = self.with(rhs);
    }
    pub fn remove(&mut self, rhs: Self) {
        *self = self.without(rhs);
    }
}
impl Room {
    pub const W: u16 = 64;
    pub const H: u16 = 64;
    const CELLS: u32 = Self::W as u32 * Self::H as u32;
    const CENTER: Coord = Coord([Self::W / 2, Self::H / 2]);
}

////////////
impl Coord {
    pub fn try_step(self, dir: Direction) -> Option<Self> {
        let Coord([x, y]) = self;
        let xy = match dir {
            Direction::Up if y > 0 => [x, y - 1],
            Direction::Left if x > 0 => [x - 1, y],
            Direction::Down if y < Room::H - 2 => [x, y + 1],
            Direction::Right if x < Room::W - 2 => [x + 1, y],
            _ => return None,
        };
        Some(Self(xy))
    }
}
impl Direction {
    pub fn invert(self) -> Self {
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
    pub fn horizontal(self) -> bool {
        match self {
            Self::Up | Self::Down => false,
            Self::Left | Self::Right => true,
        }
    }
}

impl Room {
    fn dir_to_random_blocked_adjacent_to(
        &self,
        rng: &mut Rng,
        src: Coord,
    ) -> Option<(Direction, Coord)> {
        let mut dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        rng.shuffle_slice(&mut dirs);
        let steps_to_blocked = move |dir: Direction| {
            if let Some(dest) = src.try_step(dir) {
                if self.get_cell(dest).superset_of(Cell::BLOCKED) {
                    return Some((dir, dest));
                }
            }
            None
        };
        dirs.iter().copied().filter_map(steps_to_blocked).next()
    }
    fn update_cells_with_step(&mut self, [src, dest]: [Coord; 2], dir: Direction) {
        use Cell as Rcd;
        self.get_mut_cell(dest).remove(Rcd::BLOCKED);
        let coord = if dir.crossed_wall_at_src() { src } else { dest };
        let flags = if dir.horizontal() { Rcd::WALL_LE } else { Rcd::WALL_UP };
        self.get_mut_cell(coord).remove(flags);
    }
    pub fn new(maybe_seed: Option<u64>) -> Self {
        use Cell as Rcd;
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
        for _ in 0..(Room::CELLS / 8) {
            let flag = if rng.gen_bool() { Rcd::WALL_LE } else { Rcd::WALL_UP };
            let coord = Coord([
                rng.fastrand_rng.u16(1..Room::W - 1), // x
                rng.fastrand_rng.u16(1..Room::H - 1),
            ]);
            me.get_mut_cell(coord).remove(flag);
        }
        me
    }
    pub fn coord_iter() -> impl Iterator<Item = Coord> {
        (0..Self::H).flat_map(|y| (0..Self::W).map(move |x| Coord([x, y])))
    }
    pub fn iter_cells(&self) -> impl Iterator<Item = (Coord, Cell)> + '_ {
        Self::coord_iter().map(move |coord| (coord, self.get_cell(coord)))
    }
    pub fn iter_walls(&self) -> impl Iterator<Item = (Coord, bool)> + '_ {
        self.iter_cells().flat_map(|(coord, cell)| {
            let some_coord = |up, some| if some { Some((coord, up)) } else { None };
            some_coord(true, cell.subset_of(Cell::WALL_UP))
                .into_iter()
                .chain(some_coord(false, cell.subset_of(Cell::WALL_LE)))
        })
    }
    pub fn get_mut_cell(&mut self, Coord([x, y]): Coord) -> &mut Cell {
        &mut self.data[y as usize][x as usize]
    }
    pub fn get_cell(&self, Coord([x, y]): Coord) -> Cell {
        self.data[y as usize][x as usize]
    }
    pub fn ascii_draw(&self) {
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        use {std::io::Write, Cell as Rcd};
        for row in self.data.iter() {
            // one row for vertical walls
            for cell in row.iter() {
                let up_char = if cell.superset_of(Rcd::WALL_UP) { '―' } else { ' ' };
                let _ = write!(stdout, "·{}{}", up_char, up_char);
            }
            let _ = writeln!(stdout);
            // one row for horizontal walls & blockages
            for cell in row.iter() {
                let left_char = if cell.superset_of(Rcd::WALL_LE) { '|' } else { ' ' };
                let blocked_char = if cell.superset_of(Rcd::BLOCKED) { '#' } else { ' ' };
                let _ = write!(stdout, "{}{}{}", left_char, blocked_char, blocked_char);
            }
            let _ = writeln!(stdout);
        }
    }
}
