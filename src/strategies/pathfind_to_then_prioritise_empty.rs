use crate::{
    game::{Brain, Cell, Dir, Pos as _},
    strategies,
};

pub fn step(
    grid: &[Cell],
    player_id: u8,
    pos: (usize, usize),
    brain: &mut Brain,
    width: usize,
    height: usize,
) -> Dir {
    if pos
        .neighbours(width, height)
        .into_iter()
        .filter(|neighbour| matches!(grid[neighbour.1 * width + neighbour.0], Cell::Empty))
        .count()
        > 0
    {
        strategies::prioritise_empty::step(grid, player_id, pos, brain, width, height)
    } else {
        strategies::pathfind_to_empty::step(grid, player_id, pos, brain, width, height)
    }
}
