use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct PrioritiseEmptyStrategy;

impl Strategy for PrioritiseEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Prioritise empty"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        let all_dirs = vec![Dir::None, Dir::Up, Dir::Down, Dir::Left, Dir::Right];
        let mut possible_dirs: Vec<Dir> = Vec::new();
        let mut priority_dirs: Vec<Dir> = Vec::new();
        let pos = player.position.expect("Player doesn't have a position");

        for dir in all_dirs {
            if !player.can_move(grid, dir, width, height) {
                continue;
            }

            let new_pos = pos + dir;

            match grid[new_pos.y * width + new_pos.x] {
                Cell::Empty => priority_dirs.push(dir),
                Cell::Player(player_id) | Cell::PlayerClaimed(player_id)
                    if player_id == player.id =>
                {
                    possible_dirs.push(dir);
                }
                _ => {}
            }
        }

        if !priority_dirs.is_empty() {
            return priority_dirs[fastrand::usize(..priority_dirs.len())];
        }

        possible_dirs[fastrand::usize(..possible_dirs.len())]
    }
}
