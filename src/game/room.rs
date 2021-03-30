use {
    crate::{
        bit_set::{self, BitIndex, BitSet},
        prelude::*,
        rng::Rng,
        Dim,
    },
    core::ops::Neg,
};

pub const ROOM_SIZE: DimMap<u32> = DimMap::new([WrapInt::DOMAIN_SIZE; 2]);
pub const CELL_COUNTS: DimMap<u8> = DimMap::new([8, 8]);
pub const TOT_CELL_COUNT: u16 = CELL_COUNTS.arr[0] as u16 * CELL_COUNTS.arr[1] as u16;
pub const CELL_SIZE: Size = Size::new([
    (ROOM_SIZE.arr[0] / CELL_COUNTS.arr[0] as u32) as u16,
    (ROOM_SIZE.arr[1] / CELL_COUNTS.arr[1] as u32) as u16,
]);
pub const HALF_CELL_SIZE: Size = CELL_SIZE.scalar_div(2);

///////////////////////////////////////////////
// # Data types

pub struct Room {
    pub wall_sets: DimMap<BitSet>,
}
#[derive(Default, Hash, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Coord {
    // invariant: top_left corner
    pos: Pos,
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
        rng.fastrand_rng.shuffle(&mut dirs);
        dirs.iter()
            .copied()
            .filter_map(move |dir| {
                self.try_visit_in_direction(src, dir).map(move |dest| (dir, dest))
            })
            .next()
    }
    fn try_visit_in_direction(&mut self, src: Coord, dir: Direction) -> Option<Coord> {
        let dest = src.stepped(dir);
        if self.visited.insert(dest.bit_index()) {
            // successfully
            let cwi = dir.crosses_wall_info();
            let coord = if cwi.managed_by_src { src } else { dest };
            self.wall_sets[cwi.dim].remove(coord.bit_index());
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
    pub fn new_seeded(seed: u64) -> (Self, Rng) {
        let mut rng = Rng::new_seeded(seed);
        let room = Self::new(&mut rng);
        (room, rng)
    }
    pub fn new(rng: &mut Rng) -> Self {
        let mut incomplete_room = IncompleteRoom {
            visited: Default::default(),
            wall_sets: DimMap::new([BitSet::full(), BitSet::full()]),
        };

        let mut at = Coord::random(rng);
        incomplete_room.visited.insert(at.bit_index());

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
            incomplete_room.wall_sets[Dim::random(rng)].remove(BitIndex::random(rng));
        }
        Room { wall_sets: incomplete_room.wall_sets }
    }
    pub fn iter_walls(&self) -> impl Iterator<Item = (Coord, Dim)> + '_ {
        let dimmed_iter =
            move |o| self.wall_sets[o].iter().map(move |bi| (Coord::from_bit_index(bi), o));
        dimmed_iter(X).chain(dimmed_iter(Y))
    }
    pub fn wall_cells_to_check_at(
        mut pos: Pos,
        wall_dim: Dim,
    ) -> impl Iterator<Item = Coord> + Clone {
        std::array::IntoIter::new(match wall_dim {
            X => {
                // H walls! search grid 3 wide and 2 high
                pos[Y] -= HALF_CELL_SIZE[Y];
                let a = Coord::from_pos(pos);
                let b = a.stepped(Down);
                /*
                [. a .]
                [. b .]
                */
                [a.stepped(Left), a, a.stepped(Right), b.stepped(Left), b, b.stepped(Right)]
            }
            Y => {
                // V walls! search grid 2 wide and 3 high
                pos[X] -= HALF_CELL_SIZE[X];
                let a = Coord::from_pos(pos);
                let b = a.stepped(Right);
                /*
                [. .]
                [a b]
                [. .]
                */
                [a.stepped(Up), b.stepped(Up), a, b, a.stepped(Down), b.stepped(Down)]
            }
        })
    }
}

impl Coord {
    #[inline]
    pub fn stepped(mut self, dir: Direction) -> Self {
        self.pos[dir.dim()] += dir.sign() * CELL_SIZE[dir.dim()] as i16;
        self
    }
    /////////////////////
    pub fn corner_pos(self) -> Pos {
        self.pos
    }
    pub fn center_pos(self) -> Pos {
        self.corner_pos() + HALF_CELL_SIZE.map(WrapInt::from)
    }
    pub fn from_pos(mut pos: Pos) -> Self {
        // FLOOR
        for dim in Dim::iter_domain() {
            let val: u16 = pos[dim].into();
            pos[dim] = (val / CELL_SIZE[dim] * CELL_SIZE[dim]).into();
        }
        Self { pos }
    }
    //////////////////////
    pub fn random(rng: &mut Rng) -> Self {
        Self::from_bit_index(BitIndex::random(rng))
    }
    pub fn from_bit_index(bit_index: BitIndex) -> Self {
        let x = bit_index.0 % CELL_COUNTS[X] as u16;
        let y = bit_index.0 / CELL_COUNTS[X] as u16;
        let x = WrapInt::from(x * CELL_SIZE[X]);
        let y = WrapInt::from(y * CELL_SIZE[Y]);
        Self { pos: Pos::new_xy(x, y) }
    }
    pub fn bit_index(self) -> BitIndex {
        let f = move |dim| Into::<u16>::into(self.pos[dim]) / CELL_SIZE[dim];
        let x = f(X);
        let y = f(Y);
        BitIndex(y * CELL_COUNTS[X] as u16 + x)
    }
}
