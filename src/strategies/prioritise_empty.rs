use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct PrioritiseEmptyStrategy;

impl Strategy for PrioritiseEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Prioritise empty"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        let pos = player.position.expect("Player doesn't have a position");

        let neighbours = pos.neighbours(width, height);

        let priority_dirs = neighbours
            .iter()
            .filter(|neighbour| matches!(grid[neighbour.y * width + neighbour.x], Cell::Empty))
            .map(|neighbour| Dir::from_to(pos, *neighbour));

        if priority_dirs.clone().count() > 0 {
            return fastrand::choice(priority_dirs.collect::<Vec<Dir>>())
                .expect("This shouldn't happen");
        }

        fastrand::choice(
            neighbours
                .into_iter()
                .map(|neighbour| Dir::from_to(pos, neighbour))
                .filter(|dir| player.can_move(grid, *dir, width, height))
                .collect::<Vec<Dir>>(),
        )
        .unwrap_or(Dir::None)
    }
}
