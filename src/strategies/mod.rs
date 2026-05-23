use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, LazyLock},
};

use crate::game::{Brain, Cell, Dir};

mod do_nothing;
mod hug_wall;
mod pathfind_to_empty;
mod pathfind_to_then_prioritise_empty;
mod prioritise_empty;
mod random_walk;
mod spiral;

pub static STRATEGIES: LazyLock<HashMap<u8, Arc<dyn Strategy>>> = LazyLock::new(|| {
    let mut map: HashMap<u8, Arc<dyn Strategy>> = HashMap::new();

    map.insert(0, Arc::new(do_nothing::DoNothingStrategy));
    map.insert(1, Arc::new(random_walk::RandomWalkStrategy));
    map.insert(2, Arc::new(prioritise_empty::PrioritiseEmptyStrategy));
    map.insert(3, Arc::new(spiral::SpiralStrategy));
    map.insert(4, Arc::new(pathfind_to_empty::PathfindToEmptyStrategy));
    map.insert(
        5,
        Arc::new(pathfind_to_then_prioritise_empty::PathfindPrioritiseEmptyStrategy),
    );
    map.insert(6, Arc::new(hug_wall::HugWallStrategy));

    map
});

pub trait Strategy: Send + Sync {
    fn get_name(&self) -> &'static str;

    fn step(
        &self,
        grid: &[Cell],
        player_id: u8,
        pos: (usize, usize),
        brain: &mut Brain,
        width: usize,
        height: usize,
    ) -> Dir;
}

impl Debug for dyn Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Strategy")
            .field(&self.get_name().to_owned())
            .finish()
    }
}
