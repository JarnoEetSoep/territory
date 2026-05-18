use crate::{
    game::{Cell, Dir, Pos},
    strategies::Strategy,
};

pub struct PathfindToEmptyStrategy;

impl Strategy for PathfindToEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Pathfind to empty"
    }

    fn step(&self, _grid: &[Cell], _width: usize, _height: usize, _pos: Pos, _id: u8) -> Dir {
        Dir::None
    }
}
