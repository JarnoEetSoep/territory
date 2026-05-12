use crate::game::{Cell, Dir, Pos};

mod random_walk;
mod do_nothing;

pub trait Strategy {
    fn get_name(&self) -> &str;

    fn step(&self, grid: &Vec<Vec<Cell>>, pos: Pos, id: u8) -> Dir;
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Strategies {
    RandomWalkStrategy,
    DoNothingStrategy
}

impl Strategies {
    pub fn list_strategies() -> Vec<Self> {
        vec![
            Self::RandomWalkStrategy,
            Self::DoNothingStrategy
        ]
    }

    pub fn get(&self) -> Box<dyn Strategy> {
        match self {
            Self::RandomWalkStrategy => Box::new(random_walk::RandomWalkStrategy),
            Self::DoNothingStrategy => Box::new(do_nothing::DoNothingStrategy)
        }
    }
}