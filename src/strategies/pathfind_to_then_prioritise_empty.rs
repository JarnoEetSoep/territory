use crate::{
    game::{Cell, Dir, Player},
    strategies::{
        Strategy, pathfind_to_empty::PathfindToEmptyStrategy,
        prioritise_empty::PrioritiseEmptyStrategy,
    },
};

pub struct PathfindPrioritiseEmptyStrategy;

impl Strategy for PathfindPrioritiseEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Pathfind prioritise empty"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        if player
            .position
            .expect("Player doesn't have a position")
            .neighbours(width, height)
            .into_iter()
            .filter(|neighbour| matches!(grid[neighbour.y * width + neighbour.x], Cell::Empty))
            .count()
            > 0
        {
            PrioritiseEmptyStrategy.step(grid, player, width, height)
        } else {
            PathfindToEmptyStrategy.step(grid, player, width, height)
        }
    }
}
