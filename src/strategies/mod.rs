use crate::game::{Cell, Dir, Pos};

mod do_nothing;
mod pathfind_to_empty;
mod prioritise_empty;
mod random_walk;

pub trait Strategy {
    fn get_name(&self) -> &'static str;

    fn step(&self, grid: &[Cell], width: usize, height: usize, pos: Pos, id: u8) -> Dir;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Strategies {
    #[default]
    DoNothingStrategy,
    RandomWalkStrategy,
    PrioritiseEmptyStrategy,
    PathfindToEmptyStrategy,
}

impl Strategies {
    pub fn list_strategies() -> Vec<Self> {
        vec![
            Self::DoNothingStrategy,
            Self::RandomWalkStrategy,
            Self::PrioritiseEmptyStrategy,
            Self::PathfindToEmptyStrategy,
        ]
    }

    pub fn get(&self) -> Box<dyn Strategy> {
        match self {
            Self::DoNothingStrategy => Box::new(do_nothing::DoNothingStrategy),
            Self::RandomWalkStrategy => Box::new(random_walk::RandomWalkStrategy),
            Self::PrioritiseEmptyStrategy => Box::new(prioritise_empty::PrioritiseEmptyStrategy),
            Self::PathfindToEmptyStrategy => Box::new(pathfind_to_empty::PathfindToEmptyStrategy),
        }
    }
}
