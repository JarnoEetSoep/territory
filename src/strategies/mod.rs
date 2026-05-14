use crate::game::{Cell, Dir, Pos};

mod do_nothing;
mod pathfind_to_empty;
mod random_walk;
mod prioritise_empty;

pub trait Strategy {
    fn get_name(&self) -> &'static str;

    fn step(&self, grid: &[Vec<Cell>], pos: Pos, id: u8) -> Dir;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Strategies {
    RandomWalkStrategy,
    DoNothingStrategy,
    PathfindToEmptyStrategy,
    PrioritiseEmptyStrategy,
}

impl Strategies {
    pub fn list_strategies() -> Vec<Self> {
        vec![
            Self::RandomWalkStrategy,
            Self::DoNothingStrategy,
            Self::PathfindToEmptyStrategy,
            Self::PrioritiseEmptyStrategy,
        ]
    }

    pub fn get(&self) -> Box<dyn Strategy> {
        match self {
            Self::RandomWalkStrategy => Box::new(random_walk::RandomWalkStrategy),
            Self::DoNothingStrategy => Box::new(do_nothing::DoNothingStrategy),
            Self::PathfindToEmptyStrategy => Box::new(pathfind_to_empty::PathfindToEmptyStrategy),
            Self::PrioritiseEmptyStrategy => Box::new(prioritise_empty::PrioritiseEmptyStrategy)
        }
    }
}
