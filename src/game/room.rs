use {
    crate::{
        bit_set::{self, BitIndex, BitSet},
        prelude::*,
        rng::Rng,
        Dim,
    },
    core::ops::Neg,
};

pub const ROOM_SIZE: DimMap<u32> = DimMap { arr: [u16::MAX as u32 + 1; 2] };
pub const CELLS: DimMap<u8> = DimMap { arr: [8, 8] };
pub const TOT_CELLS: u16 = CELLS.arr[0] as u16 * CELLS.arr[1] as u16;
pub const CELL_SIZE: DimMap<u16> = DimMap {
    arr: [
        (ROOM_SIZE.arr[0] / CELLS.arr[0] as u32) as u16,
        (ROOM_SIZE.arr[1] / CELLS.arr[1] as u32) as u16,
    ],
};

pub const HALF_CELL_SIZE: DimMap<u16> =
    DimMap { arr: [CELL_SIZE.arr[0] / 2, CELL_SIZE.arr[1] / 2] };

///////////////////////////////////////////////
// # Data types

pub struct Room {
    pub teleporters: BitSet,
    pub wall_sets: DimMap<BitSet>,
}
#[derive(Default, Hash, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Coord {
    map: DimMap<u8>,
}
struct IncompleteRoom {
    wall_sets: DimMap<BitSet>,
    visited: BitSet,
}
struct CrossesWallInfo {
    dim: Dim,
    managed_by_src: bool,
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
        self.wall_sets.arr.iter().map(|bitset| bitset.len() as u32).sum()
    }
    pub fn new(rng: &mut Rng) -> Self {
        let mut incomplete_room = IncompleteRoom {
            visited: Default::default(),
            wall_sets: DimMap { arr: [BitSet::full(), BitSet::full()] },
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
                .map(move |_| Into::<BitIndex>::into(Coord::random(rng)))
                .collect(),
        }
    }
    pub fn iter_walls(&self) -> impl Iterator<Item = (Coord, Dim)> + '_ {
        let dimmed_iter = move |o| self.wall_sets[o].iter().map(move |i| (i.into(), o));
        dimmed_iter(X).chain(dimmed_iter(Y))
    }
}

impl Coord {
    pub fn check_for_collisions_at(
        wall_dim: Dim,
        mut v: DimMap<WrapInt>,
    ) -> impl Iterator<Item = Self> + Clone {
        let tl = {
            v[!wall_dim] += CELL_SIZE[!wall_dim];
            Self::from_vec2_floored(v)
        };
        let dims = match wall_dim {
            X => [3, 2],
            Y => [2, 3],
        };
        (0..dims[0]).flat_map(move |x| {
            (0..dims[1]).map(move |y| {
                Self::new([(tl.map[X] + x).wrapping_sub(1), (tl.map[Y] + y).wrapping_sub(1)])
            })
        })
    }
    pub fn random(rng: &mut Rng) -> Self {
        BitIndex::random(rng).into()
    }
    pub fn new([x, y]: [u8; 2]) -> Self {
        let mut me = Self::default();
        me.map[X] = x % CELLS[X];
        me.map[Y] = y % CELLS[Y];
        me
    }
    pub fn stepped(self, dir: Direction) -> Self {
        let dim = dir.dim();
        let mut vec2 = self.into_vec2_corner();
        vec2[dim] += CELL_SIZE[dim];
        Self::from_vec2_floored(vec2)
    }
    pub fn into_vec2_center(self) -> DimMap<WrapInt> {
        self.into_vec2_corner() + HALF_CELL_SIZE.map(|value| WrapInt::from(value))
    }
    //
    pub fn from_vec2_floored(pos: DimMap<WrapInt>) -> Self {
        Self { map: pos.kv_map(|dim, &wi| (Into::<u16>::into(wi) / CELL_SIZE[dim]) as u8) }
    }
    pub fn into_vec2_corner(self) -> DimMap<WrapInt> {
        self.map.kv_map(|dim, &value| WrapInt::from(value as u16 * CELL_SIZE[dim]))
    }
}

impl Into<BitIndex> for Coord {
    fn into(self) -> BitIndex {
        BitIndex(self.map[Y] as u16 * CELLS[X] as u16 + self.map[X] as u16)
    }
}

impl Into<Coord> for BitIndex {
    fn into(self) -> Coord {
        let arr = [(self.0 % CELLS[Y] as u16) as u8, (self.0 / CELLS[X] as u16) as u8];
        Coord { map: DimMap { arr } }
    }
}
