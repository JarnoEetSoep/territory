#![allow(clippy::all)]

#[allow(warnings)]
mod bindings;
use bindings::*;

use game::strategy_plugin::{logger, random};

struct RandomWalkStrategy;

impl Guest for RandomWalkStrategy {
    fn get_name() -> String {
        "Random walk".to_owned()
    }

    fn step(
        grid: Vec<Cell>,
        player_id: u8,
        pos: (u32, u32),
        _brain: Brain,
        width: u32,
        height: u32,
    ) -> Action {
        logger::debug("Random Walk stepping");

        let dirs = pos
            .neighbours(width, height)
            .into_iter()
            .map(|neighbour| Dir::from_to(pos, neighbour))
            .filter(|dir| pos.can_move(&grid, *dir, width, height, player_id))
            .collect::<Vec<Dir>>();

        if dirs.len() == 0 {
            return Dir::None.into();
        }

        let idx = random::from_vec(&(0..dirs.len() as u32).collect::<Vec<u32>>()).unwrap();

        dirs[idx as usize].into()
    }
}

export!(RandomWalkStrategy);
