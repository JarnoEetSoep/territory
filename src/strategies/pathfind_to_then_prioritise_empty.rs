use crate::{
    game::{Brain, Cell, Dir, Pos},
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

    fn step(
        &self,
        grid: &[Cell],
        player_id: u8,
        pos: Pos,
        brain: &mut Brain,
        width: usize,
        height: usize,
    ) -> Dir {
        if pos
            .neighbours(width, height)
            .into_iter()
            .filter(|neighbour| matches!(grid[neighbour.y * width + neighbour.x], Cell::Empty))
            .count()
            > 0
        {
            PrioritiseEmptyStrategy.step(grid, player_id, pos, brain, width, height)
        } else {
            PathfindToEmptyStrategy.step(grid, player_id, pos, brain, width, height)
        }
    }
}
