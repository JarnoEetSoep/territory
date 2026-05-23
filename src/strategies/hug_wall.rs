use crate::{
    game::{Brain, Cell, Dir, Pos as _},
    strategies::{Strategy, spiral::SpiralStrategy},
};

const GO_RIGHT: u8 = 0;
const SPIRAL: u8 = 1;

pub struct HugWallStrategy;

impl Strategy for HugWallStrategy {
    fn get_name(&self) -> &'static str {
        "Hug wall"
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
        if brain.memory.is_empty() {
            brain.memory.push(GO_RIGHT);
        }

        if brain.memory[0] == GO_RIGHT {
            if !pos.can_move(grid, Dir::Right, width, height, player_id) {
                brain.memory[0] = SPIRAL;
                brain.facing = Dir::Up;

                return SpiralStrategy.step(grid, player_id, pos, brain, width, height);
            }

            Dir::Right
        } else {
            SpiralStrategy.step(grid, player_id, pos, brain, width, height)
        }
    }
}
