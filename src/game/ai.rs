use {
    crate::{
        bit_set::{BitIndex, FullBitIndexMap, INDICES},
        game::{rendering::VIEW_SIZE, Coord, PlayerColor, Room, World, PLAYER_SIZE, ZERO_POS},
        prelude::*,
    },
    std::collections::{HashMap, VecDeque},
};

////////////////////////////////////////////////////////////
// #[derive(Debug)]
// pub struct BeelineAi {
//     // Makes a beeline for its prey through the maze,
//     // ignoring everything else
//     my_color: PlayerColor, // final
//     prey_to_me: Vec<Coord>,
//     prey_at_guess: Coord,
// }

struct ShortestPaths {
    map: HashMap<[Coord; 2], u16>,
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
// fn form_a_line([a, b, c]: [Pos; 3]) -> bool {
//     Dim::iter_domain().any(|dim| a[dim] == b[dim] && b[dim] == c[dim])
// }
fn coord_dist([a, b]: [Coord; 2]) -> Size {
    (a.corner_pos() - b.corner_pos()).distances_from_zero()
}
fn pos_of_player(world: &World, color: PlayerColor) -> Pos {
    world.entities.players[color].pos
}
fn coord_of_player(world: &World, color: PlayerColor) -> Coord {
    Coord::from_pos_flooring(pos_of_player(world, color))
}
// fn dir_to_vel(dir: Direction) -> Vel {
//     let mut vel = Vel::default();
//     vel[dir.dim()] = Some(dir.sign());
//     vel
// }
// impl AiExt for BeelineAi {
//     fn new(my_color: PlayerColor, rng: &mut Rng) -> Box<dyn Ai> {
//         Box::new(Self { prey_to_me: vec![], my_color, prey_at_guess: Coord::random(rng) })
//     }
// }
// impl Ai for BeelineAi {
//     fn i_was_moved(&mut self) {
//         self.prey_to_me.clear();
//     }
//     fn update(&mut self, world: &World, rng: &mut Rng) -> Vel {
//         if rng.gen_bits(self.prey_to_me.len().min(11) as u8 + 2) == 0 {
//             self.remodel(world, rng);
//         }
//         let me_at = world.entities.players[self.my_color].pos;
//         while let Some(coord) = self.prey_to_me.last() {
//             let dest = coord.center_pos();
//             let diff = dest - me_at;
//             if diff == ZERO_POS {
//                 // consider this coord reached. move toward the next!
//                 self.prey_to_me.pop();
//             } else {
//                 // have not yet reached this coord. move toward it!
//                 return diff_to_vel(diff);
//             }
//         }
//         // nowhere to go
//         Vel::default()
//     }
// }
// impl BeelineAi {
//     #[inline]
//     fn update_prey_at_guess(&mut self, world: &World, rng: &mut Rng) {
//         let me_at = coord_of_player(world, self.my_color);
//         let prey_color = self.my_color.prey();
//         if coord_dist([coord_of_player(world, prey_color), me_at])
//             < VIEW_SIZE + PLAYER_SIZE.scalar_div(2)
//         {
//             // You're close => I can see you
//             self.prey_at_guess = coord_of_player(world, prey_color);
//         } else if coord_dist([self.prey_at_guess, me_at]) < VIEW_SIZE.scalar_div(2) {
//             // You're not close, but I am close to where I guessed => you're NOT here
//             self.prey_at_guess = Coord::random(rng);
//         }
//     }
//     fn remodel(&mut self, world: &World, rng: &mut Rng) {
//         // FOR NOW: just beeline for my prey
//         // 1. where am I?
//         let me_at = coord_of_player(world, self.my_color);
//         // 2. update my views of other players
//         self.update_prey_at_guess(world, rng);
//         // build path of coords from prey to me.
//         self.prey_to_me.clear();
//         // find shortest route from me_at to prey_at
//         let towards_me = {
//             // BFS from me_at to to prey_at
//             let mut towards_me =
//                 HashMap::<Coord, Direction>::with_capacity(TOT_CELL_COUNT as usize);
//             let mut bfs_queue = std::collections::VecDeque::with_capacity(TOT_CELL_COUNT as usize);
//             let mut at = me_at;
//             bfs_queue.push_back(at);
//             while at != self.prey_at_guess {
//                 for step_dir in Direction::iter_domain() {
//                     if let Some(dest) = at.stepped_in_room(&world.room, step_dir) {
//                         // coord C visited IFF towards_prey maps key C
//                         towards_me.entry(dest).or_insert_with(|| {
//                             bfs_queue.push_back(dest);
//                             -step_dir
//                         });
//                     }
//                 }
//                 at = bfs_queue.pop_front().unwrap();
//             }
//             towards_me
//         };
//         let mut at = self.prey_at_guess;
//         loop {
//             self.prey_to_me.push(at);
//             if at == me_at {
//                 if let [.., a, b] = self.prey_to_me.as_slice() {
//                     if form_a_line([
//                         a.center_pos(),
//                         b.center_pos(),
//                         world.entities.players[self.my_color].pos,
//                     ]) {
//                         self.prey_to_me.pop();
//                     }
//                 };
//                 break;
//             }
//             at = at.stepped(towards_me[&at]);
//         }
//     }
// }
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
            // hunting prey
            // println!("hunting prey");
            Self::sink_toward(me_at, &world.room, &self.sink_to_prey)
        } else {
            // fleeing predator
            // println!("fleeing predator");
            if self.sink_to_teleporters[bi] < self.sink_to_predator[bi] {
                // I can make it!
                // println!("... to teleporter");
                Self::sink_toward(me_at, &world.room, &self.sink_to_teleporters)
            } else {
                // I can't make it!
                // println!("... away");
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

// fn whose_turn(my_color: PlayerColor, depth: usize) -> PlayerColor {
//     match depth % 3 {
//         0 => my_color,
//         1 => my_color.prey(),
//         2 | _ => my_color.predator(),
//     }
// }
// fn first_dir() -> Direction {
//     Direction::Up
// }
// fn next_dir_if_possible(dir: Direction) -> Option<Direction> {
//     Some(match dir {
//         Up => Down,
//         Down => Left,
//         Left => Right,
//         Right => return None,
//     })
// }

// struct DistLookup {}
// impl DistLookup {
//     fn dist(&self, [a, b]: [Coord; 2]) -> u16 {
//         todo!()
//     }
// }

// struct RecRet {
//     dist_to_preys: PlayerDists,
//     next_step: Option<Direction>,
// }
// type PlayerDists = PlayerArr<u16>;
// fn minimax(
//     dist_lookup: &DistLookup,
//     world: &World,
//     coords: PlayerArr<Coord>,
//     col: PlayerColor,
//     depth_to_go: u8,
// ) -> RecRet {
//     let mut best = RecRet {
//         next_step: None,
//         dist_to_preys: [
//             dist_lookup.dist([coords[col], coords[col.prey()]]),
//             dist_lookup.dist([coords[col.prey()], coords[col.predator()]]),
//             dist_lookup.dist([coords[col.predator()], coords[col]]),
//         ],
//     };
//     let dist_h = |dist_to_preys: &PlayerDists| {
//         dist_to_preys[col] as i32 - dist_to_preys[col.predator()] as i32
//     };
//     let mut update = |dist_to_preys: PlayerDists, next_step: Direction| {
//         if dist_h(&dist_to_preys) > dist_h(&best.dist_to_preys) {
//             best.dist_to_preys = dist_to_preys;
//             best.next_step = Some(next_step);
//         }
//     };
//     for dir in Direction::iter_domain() {
//         if let Some(dest) = coords[my_color].stepped_in_room(&world.room, dir) {
//             // overwrite player dists for my_color<->my_color.prey() and my_color<->my_color.predator()

//             coords[my_color] = dest;
//             if coords[my_color] == coords[my_color.prey()] {
//                 return RecRet { heuristics: i8::MAX, dir };
//             } else if coords[my_color] == coords[my_color.predator()] {
//                 update(RecRet { dir, heuristic: i8::MIN + 1 });
//             } else if let Some(new_depth) = depth_to_go.checked_sub(1) {
//                 // deeper
//                 update(minimax(dist_lookup, world, coords, my_color.prey(), new_depth));
//             }
//         }
//     }
//     best
// }
