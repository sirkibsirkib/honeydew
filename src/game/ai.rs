use crate::game::PrettyPos;
use crate::{
    game::{Coord, PlayerColor, World, MOVE_SIZE, TOT_CELL_COUNT},
    prelude::*,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Ai {
    my_color: PlayerColor,
    prey_to_me: Vec<Coord>,
}
fn diff_to_vel(diff: Pos) -> Vel {
    let mut vel = Vel::default();
    let dim = if diff[X].distance_from_zero() > diff[Y].distance_from_zero() { X } else { Y };
    vel[dim] = diff[dim].sign();
    vel
}
impl Ai {
    pub fn new(my_color: PlayerColor) -> Self {
        Self { prey_to_me: vec![], my_color }
    }
    fn remodel(&mut self, world: &World) {
        // FOR NOW: just beeline for my prey
        // 1. where am I?
        let coord_of_player =
            |color: PlayerColor| Coord::from_pos_flooring(world.entities.players[color].pos);
        let me_at = coord_of_player(self.my_color);
        let predator_at = coord_of_player(self.my_color.predator());
        let prey_at = coord_of_player(self.my_color.prey());

        // build path of coords from prey to me.
        self.prey_to_me.clear();
        // find shortest route from me_at to prey_at
        let towards_me = {
            // BFS from me_at to to prey_at
            let mut towards_me =
                HashMap::<Coord, Direction>::with_capacity(TOT_CELL_COUNT as usize);
            let mut bfs_queue = std::collections::VecDeque::with_capacity(TOT_CELL_COUNT as usize);
            let mut at = me_at;
            bfs_queue.push_back(at);
            while at != prey_at {
                if predator_at != at {
                    for step_dir in Direction::iter_domain() {
                        if let Some(dest) = at.stepped_in_room(&world.room, step_dir) {
                            // coord C visited IFF towards_prey maps key C

                            towards_me.entry(dest).or_insert_with(|| {
                                bfs_queue.push_back(dest);
                                -step_dir
                            });
                        }
                    }
                }
                at = bfs_queue.pop_front().unwrap();
            }
            towards_me
        };
        //
        let mut at = prey_at;
        loop {
            self.prey_to_me.push(at);
            if at == me_at {
                break;
            } else {
                at = at.stepped(towards_me[&at]);
            }
        }
    }
    pub fn update(&mut self, world: &World, _rng: &mut Rng) -> Vel {
        if self.prey_to_me.is_empty() {
            self.remodel(world);
        }
        println!("\nprey_to_me: {:?}", &self.prey_to_me);
        let me_at = world.entities.players[self.my_color].pos;
        while let Some(coord) = self.prey_to_me.last() {
            let dest = coord.center_pos();
            println!("at {:?} target {:?}", PrettyPos { pos: me_at }, PrettyPos { pos: dest });
            let diff = dest - me_at;
            println!("diff {:?}", PrettyPos { pos: diff });
            let dist = diff.distances_from_zero();
            let diff_thresh = MOVE_SIZE.scalar_mul(3);
            print!("diff abs is {:?}. diff thresh is {:?}\n", dist.arr, diff_thresh.arr);
            if dist < diff_thresh {
                // consider this coord reached. move toward the next!
                println!("within thresh. consider this target reached...");
                self.prey_to_me.pop();
                println!("prey_to_me: {:?}", &self.prey_to_me);
            } else {
                // have not yet reached this coord. move toward it!
                println!("Target NOT reached! set a course: {:?}", diff_to_vel(diff));
                return diff_to_vel(diff);
            }
        }
        println!("nowhere to go!");
        // nowhere to go
        Vel::default()
    }
}
