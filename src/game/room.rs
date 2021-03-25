// # Imports and constants

use {
    crate::{
        bit_set::{self, BitSet, Index},
        prelude::*,
        rng::Rng,
        wrap_fields::WrapVec2,
        Dim,
    },
    core::ops::Neg,
};
pub const ROOM_DIMS: [u8; 2] = [8; 2]; // for best results: keep elements power of two
pub const CELL_SIZE: [u16; 2] = [u16_nth(ROOM_DIMS[0] as u16), u16_nth(ROOM_DIMS[1] as u16)];

///////////////////////////////////////////////
// # Data types

pub struct Room {
    pub teleporters: BitSet,
    pub wall_sets: EnumMap<Dim, BitSet>,
}
#[derive(Hash, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Coord {
    x: u8, // invariant: < W
    y: u8, // invariant: < H
}
struct IncompleteRoom {
    wall_sets: EnumMap<Dim, BitSet>,
    visited: BitSet,
}
struct CrossesWallInfo {
    dim: Dim,
    managed_by_src: bool,
}

//////////////////
// helper funcs
pub const fn u16_nth(n: u16) -> u16 {
    ((u16::MAX as u32 + 1) / (n as u32)) as u16
}
/////////////////////

impl Direction {
    fn crosses_wall_info(self) -> CrossesWallInfo {
        CrossesWallInfo {
            dim: !self.dim(),
            managed_by_src: match self {
                Down | Right => false,
                Up | Left => true,
            },
        }
    }
}
impl Neg for Direction {
    type Output = Self;
    fn neg(self) -> <Self as Neg>::Output {
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

impl IncompleteRoom {
    fn try_visit_from(&mut self, rng: &mut Rng, src: Coord) -> Option<(Direction, Coord)> {
        let mut dirs = [Up, Down, Left, Right];
        rng.shuffle_slice(&mut dirs);
        dirs.iter()
            .copied()
            .filter_map(move |dir| {
                self.try_visit_in_direction(src, dir).map(move |dest| (dir, dest))
            })
            .next()
    }
    fn try_visit_in_direction(&mut self, src: Coord, dir: Direction) -> Option<Coord> {
        let dest = src.stepped(dir);
        if self.visited.insert(dest.into()) {
            // successfully
            let cwi = dir.crosses_wall_info();
            let coord = if cwi.managed_by_src { src } else { dest };
            self.wall_sets[cwi.dim].remove(coord.into());
            Some(dest)
        } else {
            None
        }
    }
}

impl Room {
    pub fn wall_count(&self) -> u32 {
        self.wall_sets.values().map(|bitset| bitset.len() as u32).sum()
    }
    pub fn new(rng: &mut Rng) -> Self {
        let mut incomplete_room = IncompleteRoom {
            visited: Default::default(),
            wall_sets: enum_map::enum_map! {
                X => BitSet::full(),
                Y =>  BitSet::full(),
            },
        };

        let mut at = Coord::random(rng);
        incomplete_room.visited.insert(at.into());

        let mut step_stack = Vec::<Direction>::with_capacity(3_000);
        loop {
            if let Some((dir, dest)) = incomplete_room.try_visit_from(rng, at) {
                at = dest;
                step_stack.push(dir);
            } else if let Some(dir) = step_stack.pop() {
                // backtrack
                at = at.stepped(-dir);
            } else {
                break;
            }
        }
        for _ in 0..(bit_set::INDICES / 4) {
            incomplete_room.wall_sets[Dim::random(rng)].remove(Coord::random(rng).into());
        }
        Room {
            wall_sets: incomplete_room.wall_sets,
            teleporters: (0..bit_set::INDICES / 16)
                .map(move |_| Into::<Index>::into(Coord::random(rng)))
                .collect(),
        }
    }
    pub fn iter_walls(&self) -> impl Iterator<Item = (Coord, Dim)> + '_ {
        let dimmed_iter = move |o| self.wall_sets[o].iter().map(move |i| (i.into(), o));
        dimmed_iter(X).chain(dimmed_iter(Y))
    }
    pub fn ascii_print(&self) {
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        use std::io::Write;
        for row_iter in Coord::iter_domain_lexicographic() {
            for coord in row_iter.clone() {
                let up_char = if self.wall_sets[X].contains(coord.into()) { '-' } else { ' ' };
                let _ = write!(stdout, "·{}{}", up_char, up_char);
            }
            let _ = writeln!(stdout);
            for coord in row_iter {
                let left_char = if self.wall_sets[Y].contains(coord.into()) { '|' } else { ' ' };
                let _ = write!(stdout, "{}  ", left_char);
            }
            let _ = writeln!(stdout);
        }
    }
}

impl Into<Coord> for Index {
    fn into(self) -> Coord {
        Coord { x: (self.0 % ROOM_DIMS[0] as u16) as u8, y: (self.0 / ROOM_DIMS[1] as u16) as u8 }
    }
}

impl Coord {
    pub fn check_for_collisions_at(
        wall_dim: Dim,
        mut v: Vec2,
    ) -> impl Iterator<Item = Self> + Clone {
        let tl = {
            v[(!wall_dim).vec_index()] += 0.5;
            Self::from_vec2_floored(v)
        };
        let dims = match wall_dim {
            X => [3, 2],
            Y => [2, 3],
        };
        (0..dims[0]).flat_map(move |x| {
            (0..dims[1])
                .map(move |y| Self::new([(tl.x + x).wrapping_sub(1), (tl.y + y).wrapping_sub(1)]))
        })
    }
    pub fn part(self, dim: Dim) -> u8 {
        match dim {
            X => self.x,
            Y => self.y,
        }
    }
    pub fn manhattan_distance(self, rhs: Self) -> u16 {
        Dim::iter_domain()
            .map(|dim| {
                let [a, b] = [self.part(dim), rhs.part(dim)];
                a.wrapping_sub(b).min(b.wrapping_sub(a)) as u16
            })
            .sum()
    }
    pub fn wall_if_stepped(mut self, dir: Direction) -> Coord {
        match dir.sign() {
            Negative => {}
            Positive => match dir.dim() {
                X => self.x = (self.x + 1) % ROOM_DIMS[0],
                Y => self.y = (self.y + 1) % ROOM_DIMS[1],
            },
        }
        self
    }
    pub fn random(rng: &mut Rng) -> Self {
        Index::random(rng).into()
    }
    pub const fn new([mut x, mut y]: [u8; 2]) -> Self {
        x %= ROOM_DIMS[0];
        y %= ROOM_DIMS[1];
        Self { x, y }
    }
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        Index::iter_domain().map(Into::into)
    }
    pub fn iter_domain_lexicographic(
    ) -> impl Iterator<Item = impl Iterator<Item = Self> + Clone> + Clone {
        (0..ROOM_DIMS[1]).map(|y| (0..ROOM_DIMS[0]).map(move |x| Coord { x, y }))
    }
    pub const fn stepped(self, dir: Direction) -> Self {
        let Self { x, y } = self;
        Self::new(match dir {
            Up => [x, y.wrapping_sub(1)],
            Down => [x, y + 1],
            Left => [x.wrapping_sub(1), y],
            Right => [x + 1, y],
        })
    }
    pub fn from_vec2_rounded(v: Vec2) -> Self {
        Self::from_vec2_floored(v + Vec2::from([0.5; 2]))
    }
    pub fn from_vec2_floored(v: Vec2) -> Self {
        Self::new([v.x as u8, v.y as u8])
    }
    pub fn into_vec2_center(self) -> Vec2 {
        self.into_vec2_corner() + Vec2::from([0.5; 2])
    }
    pub fn into_vec2_corner(self) -> Vec2 {
        Vec2::from([self.x as f32, self.y as f32])
    }
}

impl Into<Index> for Coord {
    fn into(self) -> Index {
        Index(self.y as u16 * ROOM_DIMS[0] as u16 + self.x as u16)
    }
}
