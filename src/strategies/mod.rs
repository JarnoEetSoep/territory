use std::{fmt::Debug, sync::LazyLock};

use crate::game::{Brain, Cell, Dir};

mod hug_wall;
mod pathfind_to_empty;
mod pathfind_to_then_prioritise_empty;
mod prioritise_empty;
mod random_walk;
mod spiral;

type StepFn = fn(
    grid: &[Cell],
    player_id: u8,
    pos: (usize, usize),
    brain: &mut Brain,
    width: usize,
    height: usize,
) -> Dir;

#[derive(Clone)]
pub struct Strategy {
    name: String,
    step: StepFn,
}

impl Strategy {
    pub fn new<T: Into<String>>(name: T, step: StepFn) -> Self {
        Self {
            name: name.into(),
            step,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn step_fn(&self) -> StepFn {
        self.step
    }
}

pub static STRATEGIES: LazyLock<Vec<Strategy>> = LazyLock::new(|| {
    vec![
        Strategy::new("Do Nothing", |_, _, _, _, _, _| Dir::None),
        Strategy::new("Random walk", random_walk::step),
        Strategy::new("Prioritise empty", prioritise_empty::step),
        Strategy::new("Pathfind to empty", pathfind_to_empty::step),
        Strategy::new(
            "Pathfind prioritise empty",
            pathfind_to_then_prioritise_empty::step,
        ),
        Strategy::new("Spiral", spiral::step),
        Strategy::new("Hug wall", hug_wall::step),
    ]
});

impl Debug for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Strategy").field(&self.name).finish()
    }
}
