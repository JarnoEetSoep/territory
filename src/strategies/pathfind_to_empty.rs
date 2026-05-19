use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct PathfindToEmptyStrategy;

impl Strategy for PathfindToEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Pathfind to empty"
    }

    fn step(&self, _grid: &[Cell], _player: &mut Player, _width: usize, _height: usize) -> Dir {
        Dir::None
    }
}
