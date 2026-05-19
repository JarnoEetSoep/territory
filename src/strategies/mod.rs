use crate::game::{Cell, Dir, Player};

mod do_nothing;
mod pathfind_to_empty;
mod prioritise_empty;
mod random_walk;
mod spiral;

pub trait Strategy {
    fn get_name(&self) -> &'static str;

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Strategies {
    #[default]
    DoNothingStrategy,
    RandomWalkStrategy,
    PrioritiseEmptyStrategy,
    AlwaysRightStrategy,
    PathfindToEmptyStrategy,
}

impl Strategies {
    pub fn list_strategies() -> Vec<Self> {
        vec![
            Self::DoNothingStrategy,
            Self::RandomWalkStrategy,
            Self::PrioritiseEmptyStrategy,
            Self::AlwaysRightStrategy,
            Self::PathfindToEmptyStrategy,
        ]
    }

    pub fn get(&self) -> Box<dyn Strategy> {
        match self {
            Self::DoNothingStrategy => Box::new(do_nothing::DoNothingStrategy),
            Self::RandomWalkStrategy => Box::new(random_walk::RandomWalkStrategy),
            Self::PrioritiseEmptyStrategy => Box::new(prioritise_empty::PrioritiseEmptyStrategy),
            Self::AlwaysRightStrategy => Box::new(spiral::SpiralStrategy),
            Self::PathfindToEmptyStrategy => Box::new(pathfind_to_empty::PathfindToEmptyStrategy),
        }
    }
}
