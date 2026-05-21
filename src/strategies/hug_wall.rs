use crate::{
    game::{Cell, Dir, Player},
    strategies::{Strategy, spiral::SpiralStrategy},
};

const GO_RIGHT: u8 = 0;
const SPIRAL: u8 = 1;

pub struct HugWallStrategy;

impl Strategy for HugWallStrategy {
    fn get_name(&self) -> &'static str {
        "Hug wall"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        if player.brain.memory.is_empty() {
            player.brain.memory.push(GO_RIGHT);
        }

        if player.brain.memory[0] == GO_RIGHT {
            if !player.can_move(grid, Dir::Right, width, height) {
                player.brain.memory[0] = SPIRAL;
                player.brain.facing = Dir::Up;

                return SpiralStrategy.step(grid, player, width, height);
            }

            Dir::Right
        } else {
            SpiralStrategy.step(grid, player, width, height)
        }
    }
}
