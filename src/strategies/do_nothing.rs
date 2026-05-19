use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct DoNothingStrategy;

impl Strategy for DoNothingStrategy {
    fn get_name(&self) -> &'static str {
        "Do nothing"
    }

    fn step(&self, _grid: &[Cell], _player: &mut Player, _width: usize, _height: usize) -> Dir {
        Dir::None
    }
}
