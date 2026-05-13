use rand::seq::IndexedRandom;

use crate::{
    game::{Cell, Dir, Pos},
    strategies::Strategy,
};

pub struct RandomWalkStrategy;

impl Strategy for RandomWalkStrategy {
    fn get_name(&self) -> &str {
        "Random walk"
    }

    fn step(&self, grid: &Vec<Vec<Cell>>, pos: Pos, id: u8) -> Dir {
        let all_dirs = vec![Dir::None, Dir::Up, Dir::Down, Dir::Left, Dir::Right];
        let mut possible_dirs: Vec<Dir> = Vec::new();

        for dir in all_dirs {
            match dir {
                Dir::None => {}
                Dir::Up => {
                    if pos.y == 0 {
                        continue;
                    }
                }
                Dir::Down => {
                    if usize::from(pos.y) == grid.len() - 1 {
                        continue;
                    }
                }
                Dir::Left => {
                    if pos.x == 0 {
                        continue;
                    }
                }
                Dir::Right => {
                    if usize::from(pos.x) == grid[0].len() - 1 {
                        continue;
                    }
                }
            }

            let new_pos = pos + dir;

            match grid[usize::from(new_pos.y)][usize::from(new_pos.x)] {
                Cell::Empty => possible_dirs.push(dir),
                Cell::Player(player_id) => {
                    if player_id == id {
                        possible_dirs.push(dir);
                    }
                }
                Cell::PlayerClaimed(player_id) => {
                    if player_id == id {
                        possible_dirs.push(dir);
                    }
                }
            }
        }

        *possible_dirs.choose(&mut rand::rng()).unwrap()
    }
}
