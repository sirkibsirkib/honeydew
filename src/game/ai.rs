use crate::game::{PlayerArr, PlayerArrExt};
use {
    crate::{
        bit_set::{BitIndex, FullBitIndexMap, INDICES},
        game::{
            rendering::VIEW_SIZE,
            room::{Coord, Room, ShortestPaths},
            PlayerColor, World, PLAYER_SIZE, ZERO_POS,
        },
        prelude::*,
    },
    std::collections::VecDeque,
};

pub struct PathLengthsAi {
    my_color: PlayerColor,
    sp: ShortestPaths,
    next_step: Coord,
}

#[derive(Debug)]
pub struct SinkAi {
    // Has a persistent judgement of the "quality" of each coord in the maze.
    my_color: PlayerColor, // final
    sink_to_teleporters: FullBitIndexMap<u8>,
    sink_to_prey: FullBitIndexMap<u8>,
    sink_to_predator: FullBitIndexMap<u8>,
    goal: Coord,
    prey_maybe_at: Coord,
    predator_maybe_at: Coord,
}
////////////////////////////////////////////////////////////
pub trait Ai {
    fn i_was_moved(&mut self, world: &World);
    fn update(&mut self, world: &World, rng: &mut Rng) -> Vel;
}
pub trait AiExt: Ai {
    fn new(my_color: PlayerColor, world: &World, rng: &mut Rng) -> Self;
}

///////////////////
fn diff_to_vel(diff: Pos) -> Vel {
    let mut vel = Vel::default();
    if diff != ZERO_POS {
        let dim = if diff[X].distance_from_zero() > diff[Y].distance_from_zero() { X } else { Y };
        vel[dim] = diff[dim].sign();
    }
    vel
}
fn coord_dist([a, b]: [Coord; 2]) -> Size {
    (a.corner_pos() - b.corner_pos()).distances_from_zero()
}
fn pos_of_player(world: &World, color: PlayerColor) -> Pos {
    world.entities.players[color].pos
}
fn coord_of_player(world: &World, color: PlayerColor) -> Coord {
    Coord::from_pos_flooring(pos_of_player(world, color))
}
impl AiExt for SinkAi {
    fn new(my_color: PlayerColor, world: &World, _rng: &mut Rng) -> Self {
        let me_at = coord_of_player(world, my_color);
        Self {
            my_color,
            sink_to_teleporters: FullBitIndexMap::new_copied(u8::MAX),
            sink_to_prey: FullBitIndexMap::new_copied(u8::MAX),
            sink_to_predator: FullBitIndexMap::new_copied(u8::MAX),
            goal: me_at,
            prey_maybe_at: me_at,
            predator_maybe_at: me_at,
        }
    }
}
impl Ai for SinkAi {
    fn i_was_moved(&mut self, world: &World) {
        self.goal = Coord::from_pos_flooring(world.entities.players[self.my_color].pos);
        for sink_map in ArrIter::new([
            &mut self.sink_to_teleporters,
            &mut self.sink_to_prey,
            &mut self.sink_to_predator,
        ]) {
            *sink_map = FullBitIndexMap::new_copied(u8::MAX)
        }
    }
    fn update(&mut self, world: &World, rng: &mut Rng) -> Vel {
        let pos_centered = {
            let pos = pos_of_player(world, self.my_color);
            let coord = Coord::from_pos_flooring(pos);
            pos == coord.center_pos()
        };
        if pos_centered {
            self.update_model_and_goal(world, rng);
        }
        diff_to_vel(self.goal.center_pos() - pos_of_player(world, self.my_color))
    }
}
impl SinkAi {
    fn coord_kernel(at: Coord, room: &Room) -> impl Iterator<Item = Coord> + '_ {
        let neighbor_coords =
            Direction::iter_domain().filter_map(move |dir| at.stepped_in_room(room, dir));
        std::iter::once(at).chain(neighbor_coords)
    }
    fn sink_toward(at: Coord, room: &Room, map: &FullBitIndexMap<u8>) -> Coord {
        Self::coord_kernel(at, room).min_by_key(|coord| map[coord.bit_index()]).unwrap()
    }
    fn sink_away(at: Coord, room: &Room, map: &FullBitIndexMap<u8>) -> Coord {
        Self::coord_kernel(at, room).max_by_key(|coord| map[coord.bit_index()]).unwrap()
    }
    fn reduce_sink_maps(mut a: FullBitIndexMap<u8>, b: FullBitIndexMap<u8>) -> FullBitIndexMap<u8> {
        for bi in BitIndex::iter_domain() {
            a[bi] = a[bi].min(b[bi]);
        }
        a
    }
    // only call if pos-centered
    fn update_model_and_goal(&mut self, world: &World, rng: &mut Rng) {
        if rng.gen_bits(4) == 0 {
            self.update_model(world, rng);
        }
        self.update_goal(world);
    }
    fn update_model(&mut self, world: &World, rng: &mut Rng) {
        let me_at = coord_of_player(world, self.my_color);
        let mut try_update_estimated_at = |color, estimated_at: &mut Coord| {
            let real_coord = coord_of_player(world, color);
            let dist = coord_dist([real_coord, me_at]);
            if dist < VIEW_SIZE + PLAYER_SIZE.scalar_div(2) {
                // I can see you!
                *estimated_at = real_coord;
            } else if dist < VIEW_SIZE.scalar_div(2) {
                // You're not here!
                *estimated_at = Coord::random(rng);
            }
        };
        try_update_estimated_at(self.my_color.prey(), &mut self.prey_maybe_at);
        try_update_estimated_at(self.my_color.predator(), &mut self.predator_maybe_at);

        // recompute pred/prey sink trees
        self.sink_to_prey = Self::sink_map_to(world, self.prey_maybe_at, true);
        self.sink_to_predator = Self::sink_map_to(world, self.predator_maybe_at, true);

        // recompute teleporter sink trees
        self.sink_to_teleporters = world
            .entities
            .teleporters
            .iter()
            .copied()
            .map(Coord::from_pos_flooring)
            .map(|coord| Self::sink_map_to(world, coord, false))
            .reduce(Self::reduce_sink_maps)
            .unwrap();
    }
    // only call if pos-centered
    fn update_goal(&mut self, world: &World) {
        let me_at = coord_of_player(world, self.my_color);
        let bi = me_at.bit_index();
        self.goal = if self.sink_to_prey[bi] <= self.sink_to_predator[bi] {
            Self::sink_toward(me_at, &world.room, &self.sink_to_prey)
        } else {
            if self.sink_to_teleporters[bi] < self.sink_to_predator[bi] {
                Self::sink_toward(me_at, &world.room, &self.sink_to_teleporters)
            } else {
                Self::sink_away(me_at, &world.room, &self.sink_to_predator)
            }
        };
    }
    fn sink_map_to(world: &World, sink: Coord, avoid_teleporters: bool) -> FullBitIndexMap<u8> {
        let mut map = FullBitIndexMap::new_copied(u8::MAX);
        let cap = INDICES.min(100) as u8;
        let mut bfs_queue = VecDeque::with_capacity(cap as usize);
        bfs_queue.push_back(sink);
        map[sink.bit_index()] = 0;
        // visiting in BFS. means that first time we visit a node WILL be the shortest path from sink
        // loop invariant: when visiting a node, its distance is known (and not u8::MAX);
        while let Some(coord) = bfs_queue.pop_front() {
            for step_dir in Direction::iter_domain() {
                if let Some(dest) = coord.stepped_in_room(&world.room, step_dir) {
                    if avoid_teleporters {
                        if world
                            .entities
                            .teleporters
                            .iter()
                            .any(|&t| dest == Coord::from_pos_flooring(t))
                        {
                            continue;
                        }
                    }
                    let unvisited = map[dest.bit_index()] == u8::MAX;
                    if unvisited {
                        map[dest.bit_index()] = map[coord.bit_index()] + 1;
                        bfs_queue.push_back(dest);
                    }
                }
            }
        }
        map
    }
}

impl Ai for PathLengthsAi {
    fn update(&mut self, world: &World, _rng: &mut Rng) -> Vel {
        let my_pos = pos_of_player(world, self.my_color);
        let already_at_next_step = { my_pos == self.next_step.center_pos() };
        if already_at_next_step {
            let me_bi = self.next_step.bit_index();
            let goal = self.new_goal(world);
            let goal_bi = goal.bit_index();
            let dist_to_goal = self.sp.coord_pair_path_dist([me_bi, goal_bi]).unwrap();
            self.next_step = Direction::iter_domain()
                .filter_map(|dir| self.next_step.stepped_in_room(&world.room, dir))
                .find(|closer_maybe| {
                    let dist_here =
                        self.sp.coord_pair_path_dist([closer_maybe.bit_index(), goal_bi]).unwrap();
                    dist_here == dist_to_goal - 1
                })
                .unwrap_or(self.next_step);
        }
        diff_to_vel(self.next_step.center_pos() - my_pos)
    }
    fn i_was_moved(&mut self, world: &World) {
        let me_at = coord_of_player(world, self.my_color);
        self.next_step = me_at;
    }
}
impl AiExt for PathLengthsAi {
    fn new(my_color: PlayerColor, world: &World, _rng: &mut Rng) -> Self {
        Self {
            next_step: coord_of_player(world, my_color),
            sp: ShortestPaths::new(&world.room),
            my_color,
        }
    }
}
impl PathLengthsAi {
    fn new_goal(&self, world: &World) -> Coord {
        let pair_dist = move |pair: [BitIndex; 2]| {
            self.sp.coord_pair_path_dist(pair).map(|x| x as i32).unwrap_or(i32::MAX)
        };
        let tele_bit_indices = [
            //  ok
            Coord::from_pos_flooring(world.entities.teleporters[0]).bit_index(),
            // Coord::from_pos_flooring(world.entities.teleporters[1]).bit_index(),
            // Coord::from_pos_flooring(world.entities.teleporters[2]).bit_index(),
            // Coord::from_pos_flooring(world.entities.teleporters[3]).bit_index(),
        ];
        let tele_dist_at = move |bi: BitIndex| {
            tele_bit_indices
                .iter()
                .map(move |&tele_bi| {
                    self.sp
                        .coord_pair_path_dist([bi, tele_bi])
                        .map(|x| x as i32)
                        .unwrap_or(i32::MAX)
                })
                .min()
                .unwrap()
        };
        let me_bi = coord_of_player(world, self.my_color).bit_index();
        let prey_bi = coord_of_player(world, self.my_color.prey()).bit_index();
        // let prey_to_tele_dist = tele_dist_at(prey_bi);
        let pred_bi = coord_of_player(world, self.my_color.predator()).bit_index();
        let h_val_of = move |test_bi| {
            let tele_dist = tele_dist_at(test_bi);
            let pred_dist = pair_dist([test_bi, pred_bi]);
            let prey_dist = pair_dist([test_bi, prey_bi]);
            let my_dist = pair_dist([test_bi, me_bi]);
            pred_dist * 2 - prey_dist * 2 - my_dist - tele_dist
        };
        // consider every coordinate in the room.
        BitIndex::iter_domain() //
            .max_by_key(|&bi| h_val_of(bi))
            .map(Coord::from_bit_index)
            .unwrap()
    }
}

pub struct MiniMaxAi {
    my_color: PlayerColor,
    next_step: Coord,
    sp: ShortestPaths,
}
impl Ai for MiniMaxAi {
    fn update(&mut self, world: &World, _rng: &mut Rng) -> Vel {
        let my_pos = pos_of_player(world, self.my_color);
        let already_at_next_step = { my_pos == self.next_step.center_pos() };
        if already_at_next_step {
            let at = self.next_step;
            let me_bi = self.next_step.bit_index();
            let goal = {
                let coords = PlayerArr::new_with(|col| coord_of_player(world, col));
                let ret = q_rec(&self.sp, world, self.my_color, coords, 3 * 3);
                ret.next_dir.map(|dir| at.stepped(dir)).unwrap_or(at)
            };
            let goal_bi = goal.bit_index();
            let dist_to_goal = self.sp.coord_pair_path_dist([me_bi, goal_bi]).unwrap();
            self.next_step = Direction::iter_domain()
                .filter_map(|dir| at.stepped_in_room(&world.room, dir))
                .find(|closer_maybe| {
                    let dist_here =
                        self.sp.coord_pair_path_dist([closer_maybe.bit_index(), goal_bi]).unwrap();
                    dist_here == dist_to_goal - 1
                })
                .unwrap_or(self.next_step);
        }
        diff_to_vel(self.next_step.center_pos() - my_pos)
    }
    fn i_was_moved(&mut self, world: &World) {
        let me_at = coord_of_player(world, self.my_color);
        self.next_step = me_at;
    }
}
impl AiExt for MiniMaxAi {
    fn new(my_color: PlayerColor, world: &World, _rng: &mut Rng) -> Self {
        Self {
            next_step: coord_of_player(world, my_color),
            sp: ShortestPaths::new(&world.room),
            my_color,
        }
    }
}
struct Ret {
    next_dir: Option<Direction>,
    end_up: PlayerArr<Coord>,
}
fn q_rec(
    sp: &ShortestPaths,
    world: &World,
    me: PlayerColor,
    coords: PlayerArr<Coord>,
    depth_to_go: u8,
) -> Ret {
    if let Some(deeper_depth) = depth_to_go.checked_sub(1) {
        // recursive case
        let at_tele = move |coord| {
            world.entities.teleporters.iter().any(|&pos| Coord::from_pos_flooring(pos) == coord)
        };
        let mut best = q_rec(sp, world, me.prey(), coords, deeper_depth);
        for dir in Direction::iter_domain() {
            if let Some(dest) = coords[me].stepped_in_room(&world.room, dir) {
                let mut new_coords = coords;
                new_coords[me] = dest;
                let cannot_continue = new_coords[me] == new_coords[me.prey()]
                    || new_coords[me] == new_coords[me.predator()]
                    || world
                        .entities
                        .teleporters
                        .iter()
                        .any(|&pos| Coord::from_pos_flooring(pos) == new_coords[me]);
                let new_end_up = if cannot_continue {
                    new_coords
                } else {
                    // recursive call
                    q_rec(sp, world, me.prey(), new_coords, deeper_depth).end_up
                };
                let new_best = {
                    let h_value = move |end_up: &PlayerArr<Coord>| {
                        if at_tele(end_up[me]) {
                            0
                        } else {
                            let me_bi = end_up[me].bit_index();
                            const AVG_DIST: u16 = 15;
                            let col_dist = |col: PlayerColor| {
                                if at_tele(end_up[col]) {
                                    AVG_DIST
                                } else {
                                    sp.coord_pair_path_dist([me_bi, end_up[col].bit_index()])
                                        .unwrap()
                                }
                            };
                            col_dist(me.predator()) - col_dist(me.prey())
                        }
                    };
                    h_value(&new_end_up) > h_value(&best.end_up)
                };
                if new_best {
                    best.next_dir = Some(dir);
                    best.end_up = new_end_up;
                }
            }
        }
        best
    } else {
        // stop condition
        Ret { end_up: coords, next_dir: None }
    }
}
