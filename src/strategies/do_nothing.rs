use crate::{
    game::{Cell, Dir, Pos},
    strategies::Strategy,
};

pub struct DoNothingStrategy;

impl Strategy for DoNothingStrategy {
    fn get_name(&self) -> &'static str {
        "Do nothing"
    }

    fn step(&self, _grid: &[Vec<Cell>], _pos: Pos, _id: u8) -> Dir {
        Dir::None
    }
}
