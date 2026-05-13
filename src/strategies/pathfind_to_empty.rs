use crate::{
    game::{Cell, Dir, Pos},
    strategies::Strategy,
};

pub struct PathfindToEmptyStrategy;

impl Strategy for PathfindToEmptyStrategy {
    fn get_name(&self) -> &str {
        "Pathfind to empty"
    }

    fn step(&self, _grid: &Vec<Vec<Cell>>, _pos: Pos, _id: u8) -> Dir {
        Dir::None
    }
}
