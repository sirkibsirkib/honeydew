use crate::game::rendering::VIEW_SIZE;
use crate::game::PrettyPos;
use crate::game::PLAYER_SIZE;
use crate::{
    game::{Coord, PlayerColor, World, MOVE_SIZE, TOT_CELL_COUNT},
    prelude::*,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Ai {
    my_color: PlayerColor,
    prey_to_me: Vec<Coord>,
    guess_prey_at: Coord,
}
fn diff_to_vel(diff: Pos) -> Vel {
    let mut vel = Vel::default();
    let dim = if diff[X].distance_from_zero() > diff[Y].distance_from_zero() { X } else { Y };
    vel[dim] = diff[dim].sign();
    vel
}
fn form_a_line([a, b, c]: [Pos; 3]) -> bool {
    Dim::iter_domain().any(|dim| a[dim] == b[dim] && b[dim] == c[dim])
}
fn coord_dist([a, b]: [Coord; 2]) -> Size {
    (a.corner_pos() - b.corner_pos()).distances_from_zero()
}
impl Ai {
    pub fn new(my_color: PlayerColor) -> Self {
        Self { prey_to_me: vec![], my_color, guess_prey_at: Coord::default() }
    }
    fn remodel(&mut self, world: &World, rng: &mut Rng) {
        // FOR NOW: just beeline for my prey
        // 1. where am I?
        let coord_of_player =
            |color: PlayerColor| Coord::from_pos_flooring(world.entities.players[color].pos);
        let me_at = coord_of_player(self.my_color);
        let prey_at = {
            let prey_col = self.my_color.prey();
            if coord_dist([coord_of_player(prey_col), me_at])
                < VIEW_SIZE + PLAYER_SIZE.scalar_mul(2)
            {
                // I can see you!
                self.guess_prey_at = coord_of_player(prey_col);
            } else if coord_dist([self.guess_prey_at, me_at]) < VIEW_SIZE.scalar_div(4) {
                // I am close to where I thought you were
                self.guess_prey_at = Coord::random(rng);
            }
            self.guess_prey_at
        };

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
                for step_dir in Direction::iter_domain() {
                    if let Some(dest) = at.stepped_in_room(&world.room, step_dir) {
                        // coord C visited IFF towards_prey maps key C

                        towards_me.entry(dest).or_insert_with(|| {
                            bfs_queue.push_back(dest);
                            -step_dir
                        });
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
                if let [.., a, b] = self.prey_to_me.as_slice() {
                    if form_a_line([
                        a.center_pos(),
                        b.center_pos(),
                        world.entities.players[self.my_color].pos,
                    ]) {
                        self.prey_to_me.pop();
                    }
                };
                break;
            }
            at = at.stepped(towards_me[&at]);
        }
    }
    pub fn i_was_moved(&mut self) {
        self.prey_to_me.clear();
    }
    pub fn update(&mut self, world: &World, rng: &mut Rng) -> Vel {
        const ZERO_POS: Pos = Pos::new([WrapInt::ZERO; 2]);
        if rng.gen_bits(self.prey_to_me.len().min(11) as u8 + 2) == 0 {
            self.remodel(world, rng);
        }
        println!("\nprey_to_me: {:?}", &self.prey_to_me);
        let me_at = world.entities.players[self.my_color].pos;
        while let Some(coord) = self.prey_to_me.last() {
            let dest = coord.center_pos();
            println!("at {:?} target {:?}", PrettyPos { pos: me_at }, PrettyPos { pos: dest });
            let diff = dest - me_at;
            println!("diff {:?}", PrettyPos { pos: diff });
            if diff == ZERO_POS {
                // consider this coord reached. move toward the next!
                println!("target reached...");
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
