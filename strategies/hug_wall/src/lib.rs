#![allow(clippy::all)]

#[allow(warnings)]
mod bindings;
use bindings::*;

use game::strategy_plugin::logger;

const GO_RIGHT: u8 = 0;
const SPIRAL: u8 = 1;

struct SpiralStrategy;

impl Guest for SpiralStrategy {
    fn get_name() -> String {
        "Hug wall".to_owned()
    }

    fn step(
        grid: Vec<Cell>,
        player_id: u8,
        pos: (u32, u32),
        mut brain: Brain,
        width: u32,
        height: u32,
    ) -> Action {
        logger::debug("Hug wall stepping");

        if brain.memory.is_empty() {
            brain.memory.push(GO_RIGHT);
        }

        if brain.memory[0] == GO_RIGHT {
            if !pos.can_move(&grid, Dir::Right, width, height, player_id) {
                brain.memory[0] = SPIRAL;
                brain.facing = Dir::Up;

                return spiral(grid, player_id, pos, brain, width, height);
            }

            Dir::Right.into()
        } else {
            spiral(grid, player_id, pos, brain, width, height)
        }
    }
}

fn spiral(
    grid: Vec<Cell>,
    player_id: u8,
    pos: (u32, u32),
    mut brain: Brain,
    width: u32,
    height: u32,
) -> Action {
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

export!(SpiralStrategy);
