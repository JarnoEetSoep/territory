use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct RandomWalkStrategy;

impl Strategy for RandomWalkStrategy {
    fn get_name(&self) -> &'static str {
        "Random walk"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        let all_dirs = vec![Dir::None, Dir::Up, Dir::Down, Dir::Left, Dir::Right];
        let mut possible_dirs: Vec<Dir> = Vec::new();

        for dir in all_dirs {
            if player.can_move(grid, dir, width, height) {
                possible_dirs.push(dir);
            }
        }

        possible_dirs[fastrand::usize(..possible_dirs.len())]
    }
}
