#![allow(clippy::all)]

#[allow(warnings)]
mod bindings;
use bindings::*;

use game::strategy_plugin::logger;

struct SpiralStrategy;

impl Guest for SpiralStrategy {
    fn get_name() -> String {
        "Spiral".to_owned()
    }

    fn step(
        grid: Vec<Cell>,
        player_id: u8,
        pos: (u32, u32),
        mut brain: Brain,
        width: u32,
        height: u32,
    ) -> Action {
        logger::debug("Spiral stepping");

        let facing = match brain.facing {
            Dir::None => Dir::Right,
            _ => brain.facing,
        };

        for dir in [facing.right(), facing, facing.left(), facing.left().left()] {
            if pos.can_move(&grid, dir, width, height, player_id)
                && matches!(
                    grid[((pos + dir).1 * width + (pos + dir).0) as usize],
                    Cell::Empty
                )
            {
                brain.facing = dir;

                return Action::new(dir, Some(brain));
            }
        }

        for dir in [facing.right(), facing, facing.left(), facing.left().left()] {
            if pos.can_move(&grid, dir, width, height, player_id) {
                brain.facing = dir;

                return Action::new(dir, Some(brain));
            }
        }

        Dir::None.into()
    }
}

export!(SpiralStrategy);
