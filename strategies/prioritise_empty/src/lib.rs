#![allow(clippy::all)]

#[allow(warnings)]
mod bindings;
use bindings::*;

use game::strategy_plugin::{logger, random};

struct PrioritiseEmptyStrategy;

impl Guest for PrioritiseEmptyStrategy {
    fn get_name() -> String {
        "Prioritise empty".to_owned()
    }

    fn step(
        grid: Vec<Cell>,
        player_id: u8,
        pos: (u32, u32),
        _brain: Brain,
        width: u32,
        height: u32,
    ) -> Action {
        logger::debug("Prioritise empty stepping");

        let neighbours = pos.neighbours(width, height);

        let priority_dirs = neighbours
            .iter()
            .filter(|neighbour| {
                matches!(
                    grid[(neighbour.1 * width + neighbour.0) as usize],
                    Cell::Empty
                )
            })
            .map(|neighbour| Dir::from_to(pos, *neighbour))
            .collect::<Vec<Dir>>();

        if priority_dirs.len() > 0 {
            let idx =
                random::from_vec(&(0..priority_dirs.len() as u32).collect::<Vec<u32>>()).unwrap();

            return priority_dirs[idx as usize].into();
        }

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

export!(PrioritiseEmptyStrategy);
