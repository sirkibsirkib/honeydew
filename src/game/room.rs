// # Imports and constants

use {
    crate::{
        basic::*,
        game::bit_set::{self, BitSet, Coord},
        rng::Rng,
        Orientation,
    },
    core::ops::Neg,
};

///////////////////////////////////////////////
// # Data types

struct IncompleteRoom {
    room: Room,
    visited: BitSet,
}
pub struct Room {
    pub wall_sets: enum_map::EnumMap<Orientation, BitSet>,
}
struct CrossesWallInfo {
    orientation: Orientation,
    managed_by_src: bool,
}

impl Direction {
    fn crosses_wall_info(self) -> CrossesWallInfo {
        CrossesWallInfo {
            orientation: !self.orientation(),
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
            Right => Left,
            Left => Right,
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
            self.room.wall_sets[cwi.orientation].remove(coord.into());
            Some(dest)
        } else {
            None
        }
    }
}

impl Room {
    pub fn wall_count(&self) -> u32 {
        self.wall_sets[Horizontal].len() as u32 + self.wall_sets[Vertical].len() as u32
    }
    pub fn new(rng: &mut Rng) -> Self {
        let mut incomplete_room = IncompleteRoom {
            visited: Default::default(),
            room: Room {
                wall_sets: enum_map::enum_map! {
                    Horizontal => BitSet::full(),
                    Vertical =>  BitSet::full(),
                },
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
            incomplete_room.room.wall_sets[Orientation::random(rng)]
                .remove(Coord::random(rng).into());
        }
        incomplete_room.room
    }
    pub fn iter_walls(&self) -> impl Iterator<Item = (Coord, Orientation)> + '_ {
        let oriented_iter = move |o| self.wall_sets[o].iter().map(move |i| (i.into(), o));
        oriented_iter(Horizontal).chain(oriented_iter(Vertical))
    }
    pub fn ascii_print(&self) {
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        use std::io::Write;
        for row_iter in Coord::iter_domain_lexicographic() {
            for coord in row_iter.clone() {
                let up_char =
                    if self.wall_sets[Horizontal].contains(coord.into()) { '-' } else { ' ' };
                let _ = write!(stdout, "·{}{}", up_char, up_char);
            }
            let _ = writeln!(stdout);
            for coord in row_iter {
                let left_char =
                    if self.wall_sets[Vertical].contains(coord.into()) { '|' } else { ' ' };
                let _ = write!(stdout, "{}  ", left_char);
            }
            let _ = writeln!(stdout);
        }
    }
}
