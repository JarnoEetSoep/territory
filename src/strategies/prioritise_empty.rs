use crate::{
    game::{Cell, Dir, Pos},
    strategies::Strategy,
};

pub struct PrioritiseEmptyStrategy;

impl Strategy for PrioritiseEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Prioritise empty"
    }

    fn step(&self, grid: &[Cell], width: usize, height: usize, pos: Pos, id: u8) -> Dir {
        let all_dirs = vec![Dir::None, Dir::Up, Dir::Down, Dir::Left, Dir::Right];
        let mut possible_dirs: Vec<Dir> = Vec::new();
        let mut priority_dirs: Vec<Dir> = Vec::new();

        for dir in all_dirs {
            match dir {
                Dir::None => {}
                Dir::Up => {
                    if pos.y == 0 {
                        continue;
                    }
                }
                Dir::Down => {
                    if pos.y == height - 1 {
                        continue;
                    }
                }
                Dir::Left => {
                    if pos.x == 0 {
                        continue;
                    }
                }
                Dir::Right => {
                    if pos.x == width - 1 {
                        continue;
                    }
                }
            }

            let new_pos = pos + dir;

            match grid[usize::from(new_pos.y) * usize::from(width) + usize::from(new_pos.x)] {
                Cell::Empty => priority_dirs.push(dir),
                Cell::Player(player_id) | Cell::PlayerClaimed(player_id) => {
                    if player_id == id {
                        possible_dirs.push(dir);
                    }
                }
            }
        }

        if !priority_dirs.is_empty() {
            return priority_dirs[fastrand::usize(..priority_dirs.len())];
        }

        possible_dirs[fastrand::usize(..possible_dirs.len())]
    }
}
