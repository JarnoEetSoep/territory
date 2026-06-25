#![allow(clippy::all)]

#[allow(warnings)]
mod bindings;
use bindings::*;

use game::strategy_plugin::logger;

struct DoNothingStrategy;

impl Guest for DoNothingStrategy {
    fn get_name() -> String {
        "Do nothing".to_owned()
    }

    fn step(
        _grid: Vec<Cell>,
        _player_id: u8,
        _pos: (u32, u32),
        _brain: Brain,
        _width: u32,
        _height: u32,
    ) -> Action {
        logger::debug("Do nothing stepping");

        Dir::None.into()
    }
}

export!(DoNothingStrategy);
