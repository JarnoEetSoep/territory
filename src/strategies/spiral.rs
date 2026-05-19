use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct SpiralStrategy;

impl Strategy for SpiralStrategy {
    fn get_name(&self) -> &'static str {
        "Spiral"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        let pos = player.position.expect("Player has no position");

        let facing = match player.brain.facing {
            Dir::None => Dir::Right,
            _ => player.brain.facing,
        };

        for dir in [facing.right(), facing, facing.left(), facing.left().left()] {
            if player.can_move(grid, dir, width, height)
                && matches!(grid[(pos + dir).y * width + (pos + dir).x], Cell::Empty)
            {
                player.brain.facing = dir;

                return dir;
            }
        }

        for dir in [facing.right(), facing, facing.left(), facing.left().left()] {
            if player.can_move(grid, dir, width, height) {
                player.brain.facing = dir;

                return dir;
            }
        }

        Dir::None
    }
}
