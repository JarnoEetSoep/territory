use crate::{
    game::{Brain, Cell, Dir, Pos},
    strategies::Strategy,
};

pub struct DoNothingStrategy;

impl Strategy for DoNothingStrategy {
    fn get_name(&self) -> &'static str {
        "Do nothing"
    }

    fn step(
        &self,
        _grid: &[Cell],
        _player_id: u8,
        _pos: Pos,
        _brain: &mut Brain,
        _width: usize,
        _height: usize,
    ) -> Dir {
        Dir::None
    }
}
