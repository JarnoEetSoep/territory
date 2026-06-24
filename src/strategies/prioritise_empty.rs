use crate::game::{Brain, Cell, Dir, Pos as _};

pub fn step(
    grid: &[Cell],
    player_id: u8,
    pos: (usize, usize),
    _brain: &mut Brain,
    width: usize,
    height: usize,
) -> Dir {
    let neighbours = pos.neighbours(width, height);

    let priority_dirs = neighbours
        .iter()
        .filter(|neighbour| matches!(grid[neighbour.1 * width + neighbour.0], Cell::Empty))
        .map(|neighbour| Dir::from_to(pos, *neighbour));

    if priority_dirs.clone().count() > 0 {
        return fastrand::choice(priority_dirs.collect::<Vec<Dir>>())
            .expect("This shouldn't happen");
    }

    fastrand::choice(
        neighbours
            .into_iter()
            .map(|neighbour| Dir::from_to(pos, neighbour))
            .filter(|dir| pos.can_move(grid, *dir, width, height, player_id))
            .collect::<Vec<Dir>>(),
    )
    .unwrap_or(Dir::None)
}
