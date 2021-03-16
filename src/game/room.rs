// # Imports and constants

use {
    crate::{rng::Rng, Orientation},
    core::fmt::{self, Debug, Formatter},
};

///////////////////////////////////////////////
// # Data types

#[derive(Copy, Clone, Debug)]
pub(crate) struct Coord(pub [u16; 2]);

#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) struct Cell {
    flags: u8,
}

pub(crate) struct Room {
    data: [[Cell; Self::W as usize]; Self::H as usize],
}
#[derive(Debug, Copy, Clone)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

///////////////////////////////////////////////
// # associated consts
impl Cell {
    pub const BLOCKED: Self = Self { flags: 0b00000001 };
    pub const WALL_UP: Self = Self { flags: 0b00000010 };
    pub const WALL_LE: Self = Self { flags: 0b00000100 };
    pub const TELEPOR: Self = Self { flags: 0b00001000 };
    ////////
    pub const OPEN: Self = Self { flags: 0b00000000 };
    pub const CLOSED: Self = Self::BLOCKED.with(Self::WALL_UP).with(Self::WALL_LE);
}
impl Room {
    pub const W: u16 = 8;
    pub const H: u16 = 8;
    const CELLS: u32 = Self::W as u32 * Self::H as u32;
    const CENTER: Coord = Coord([Self::W / 2, Self::H / 2]);
}

///////////////////////////////////////////////
// # methods and functions

impl Cell {
    const fn orientation_to_wall_flag(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Horizontal => Self::WALL_UP,
            Orientation::Vertical => Self::WALL_LE,
        }
    }
    pub const fn count_walls(self) -> u8 {
        match [self.has_wall(Orientation::Vertical), self.has_wall(Orientation::Horizontal)] {
            [false, false] => 0,
            [false, true] | [true, false] => 1,
            [true, true] => 2,
        }
    }
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
    pub const fn has_wall(self, orientation: Orientation) -> bool {
        self.superset_of(Self::orientation_to_wall_flag(orientation))
    }
    pub const fn is_blocked(self) -> bool {
        self.superset_of(Self::BLOCKED)
    }
    ////////
    pub fn add(&mut self, rhs: Self) {
        *self = self.with(rhs);
    }
    pub fn remove(&mut self, rhs: Self) {
        *self = self.without(rhs);
    }
    pub fn remove_wall(&mut self, orientation: Orientation) {
        self.remove(Self::orientation_to_wall_flag(orientation))
    }
}

////////////
impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_set()
            .entries(
                if self.superset_of(Self::BLOCKED) { Some("BLOCKED") } else { None }
                    .into_iter()
                    .chain(if self.superset_of(Self::WALL_UP) { Some("WALL_UP") } else { None })
                    .chain(if self.superset_of(Self::WALL_LE) { Some("WALL_LE") } else { None })
                    .chain(if self.superset_of(Self::TELEPOR) { Some("TELEPOR") } else { None }),
            )
            .finish()
    }
}
impl Coord {
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        (0..Room::H).flat_map(|y| (0..Room::W).map(move |x| Self([x, y])))
    }
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
    pub fn crosses_walls_oriented(self) -> Orientation {
        match self {
            Self::Up | Self::Down => Orientation::Horizontal,
            Self::Left | Self::Right => Orientation::Vertical,
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
        self.get_mut_cell(coord).remove_wall(dir.crosses_walls_oriented());
    }
    pub fn new(maybe_seed: Option<u64>) -> Self {
        use Cell as Rcd;
        let rng = &mut Rng::new(maybe_seed);
        let mut me = Self { data: [[Rcd::CLOSED; Self::W as usize]; Self::H as usize] };
        for row in me.data.iter_mut() {
            // in every cell, rightmost cell has no UP wall
            row.last_mut().unwrap().remove(Cell::WALL_UP)
        }
        for cell in me.data.last_mut().unwrap() {
            // in last row, every cell has no LEFT wall
            cell.remove(Cell::WALL_LE)
        }

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
    pub fn iter_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        self.data.iter().flatten().copied()
    }
    pub fn iter_walls(&self) -> impl Iterator<Item = (Coord, Orientation)> + '_ {
        Coord::iter_domain().zip(self.iter_cells()).flat_map(|(coord, cell)| {
            Orientation::iter_domain().filter_map(move |orientation| {
                if cell.has_wall(orientation) {
                    Some((coord, orientation))
                } else {
                    None
                }
            })
        })
    }
    pub fn get_mut_cell(&mut self, Coord([x, y]): Coord) -> &mut Cell {
        &mut self.data[y as usize][x as usize]
    }
    pub fn get_cell(&self, Coord([x, y]): Coord) -> Cell {
        self.data[y as usize][x as usize]
    }
    pub fn ascii_print(&self) {
        // for c in self.iter_cells() {
        //     println!("{:?}", c);
        // }
        // for c in self.iter_walls() {
        //     println!("{:?}", c);
        // }
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        use std::io::Write;
        for row in self.data.iter() {
            // one row for vertical walls
            for cell in row.iter() {
                let up_char = if cell.has_wall(Orientation::Horizontal) { '-' } else { ' ' };
                let _ = write!(stdout, "·{}{}", up_char, up_char);
            }
            let _ = writeln!(stdout);
            // one row for horizontal walls & blockages
            for cell in row.iter() {
                let left_char = if cell.has_wall(Orientation::Vertical) { '|' } else { ' ' };
                let blocked_char = if cell.is_blocked() { '#' } else { ' ' };
                let _ = write!(stdout, "{}{}{}", left_char, blocked_char, blocked_char);
            }
            let _ = writeln!(stdout);
        }
    }
    // pub fn ascii_print2(&self) {
    //     println!();
    //     let mut ascii_chars = [[b' '; Self::W as usize * 3]; Self::H as usize * 2];
    //     let stdout = std::io::stdout();
    //     let mut stdout = stdout.lock();
    //     use std::io::Write;

    //     for (coord, orientation) in self.iter_walls() {
    //         let [x, y] = [coord.0[0] as usize, coord.0[1] as usize];
    //         match orientation {
    //             Orientation::Horizontal => {
    //                 ascii_chars[y * 2][x * 3 + 1] = b'-';
    //                 ascii_chars[y * 2][x * 3 + 2] = b'-';
    //             }
    //             Orientation::Vertical => {
    //                 ascii_chars[y * 2 + 1][x * 3 + 0] = b'|';
    //             }
    //         }
    //     }

    //     for row in ascii_chars.iter() {
    //         let _ = writeln!(stdout, "{}", std::str::from_utf8(row).unwrap());
    //     }
    // }
}
