use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct RandomWalkStrategy;

impl Strategy for RandomWalkStrategy {
    fn get_name(&self) -> &'static str {
        "Random walk"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        let pos = player.position.expect("Player doesn't have a position");

        fastrand::choice(
            pos.neighbours(width, height)
                .into_iter()
                .map(|neighbour| Dir::from_to(pos, neighbour))
                .filter(|dir| player.can_move(grid, *dir, width, height))
                .collect::<Vec<Dir>>(),
        )
        .unwrap_or(Dir::None)
    }
}
