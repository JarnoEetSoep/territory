use crate::{
    game::{Brain, Cell, Dir, Pos as _},
    strategies::Strategy,
};

pub struct SpiralStrategy;

impl Strategy for SpiralStrategy {
    fn get_name(&self) -> &'static str {
        "Spiral"
    }

    fn step(
        &self,
        grid: &[Cell],
        player_id: u8,
        pos: (usize, usize),
        brain: &mut Brain,
        width: usize,
        height: usize,
    ) -> Dir {
        let facing = match brain.facing {
            Dir::None => Dir::Right,
            _ => brain.facing,
        };

        for dir in [facing.right(), facing, facing.left(), facing.left().left()] {
            if pos.can_move(grid, dir, width, height, player_id)
                && matches!(grid[(pos + dir).1 * width + (pos + dir).0], Cell::Empty)
            {
                brain.facing = dir;

                return dir;
            }
        }

        for dir in [facing.right(), facing, facing.left(), facing.left().left()] {
            if pos.can_move(grid, dir, width, height, player_id) {
                brain.facing = dir;

                return dir;
            }
        }

        Dir::None
    }
}
