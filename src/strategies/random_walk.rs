use crate::game::{Brain, Cell, Dir, Pos as _};

pub fn step(
    grid: &[Cell],
    player_id: u8,
    pos: (usize, usize),
    _brain: &mut Brain,
    width: usize,
    height: usize,
) -> Dir {
    fastrand::choice(
        pos.neighbours(width, height)
            .into_iter()
            .map(|neighbour| Dir::from_to(pos, neighbour))
            .filter(|dir| pos.can_move(grid, *dir, width, height, player_id))
            .collect::<Vec<Dir>>(),
    )
    .unwrap_or(Dir::None)
}
