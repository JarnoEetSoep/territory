#![allow(clippy::all)]

#[allow(warnings)]
mod bindings;
use bindings::*;

use std::collections::{HashMap, VecDeque};

use game::strategy_plugin::logger;

struct PathfindToEmptyStrategy;

impl Guest for PathfindToEmptyStrategy {
    fn get_name() -> String {
        "Pathfind to empty".to_owned()
    }

    fn step(
        grid: Vec<Cell>,
        player_id: u8,
        pos: (u32, u32),
        _brain: Brain,
        width: u32,
        height: u32,
    ) -> Action {
        logger::debug("Pathfind to empty stepping");

        let mut target = None;

        let mut queue = VecDeque::new();
        let mut visited = vec![false; grid.len()];
        let mut parent = HashMap::new();

        queue.push_back(pos);
        visited[(pos.1 * width + pos.0) as usize] = true;

        while let Some(pos) = queue.pop_front()
            && target.is_none()
        {
            for neighbour in pos.neighbours(width, height) {
                if visited[(neighbour.1 * width + neighbour.0) as usize] {
                    continue;
                }

                parent.insert(neighbour, pos);

                match grid[(neighbour.1 * width + neighbour.0) as usize] {
                    Cell::Empty => target = Some(neighbour),
                    Cell::PlayerClaimed(id) if id == player_id => {
                        visited[(neighbour.1 * width + neighbour.0) as usize] = true;

                        queue.push_back(neighbour);
                    }
                    _ => {}
                }
            }
        }

        if let Some(target_position) = target {
            let mut path = vec![target_position];
            let mut current = target_position;

            while let Some(parent_position) = parent.get(&current) {
                path.push(*parent_position);
                current = *parent_position;
            }

            path.reverse();

            return Dir::from_to(path[0], path[1]).into();
        }

        Dir::None.into()
    }
}

export!(PathfindToEmptyStrategy);
