#![allow(clippy::all)]

#[allow(warnings)]
mod bindings;
use bindings::*;

use std::collections::{HashMap, VecDeque};

use game::strategy_plugin::{logger, random};

struct PathfindPrioritiseEmptyStrategy;

impl Guest for PathfindPrioritiseEmptyStrategy {
    fn get_name() -> String {
        "Pathfind prioritise empty".to_owned()
    }

    fn step(
        grid: Vec<Cell>,
        player_id: u8,
        pos: (u32, u32),
        brain: Brain,
        width: u32,
        height: u32,
    ) -> Action {
        logger::debug("Pathfind prioritise empty stepping");

        if pos
            .neighbours(width, height)
            .into_iter()
            .filter(|neighbour| {
                matches!(
                    grid[(neighbour.1 * width + neighbour.0) as usize],
                    Cell::Empty
                )
            })
            .count()
            > 0
        {
            prioritise_empty(grid, player_id, pos, brain, width, height)
        } else {
            pathfind_to_empty(grid, player_id, pos, brain, width, height)
        }
    }
}

fn prioritise_empty(
    grid: Vec<Cell>,
    player_id: u8,
    pos: (u32, u32),
    _brain: Brain,
    width: u32,
    height: u32,
) -> Action {
    let neighbours = pos.neighbours(width, height);

    let priority_dirs = neighbours
        .iter()
        .filter(|neighbour| {
            matches!(
                grid[(neighbour.1 * width + neighbour.0) as usize],
                Cell::Empty
            )
        })
        .map(|neighbour| Dir::from_to(pos, *neighbour))
        .collect::<Vec<Dir>>();

    if priority_dirs.len() > 0 {
        let idx = random::from_vec(&(0..priority_dirs.len() as u32).collect::<Vec<u32>>()).unwrap();

        return priority_dirs[idx as usize].into();
    }

    let dirs = pos
        .neighbours(width, height)
        .into_iter()
        .map(|neighbour| Dir::from_to(pos, neighbour))
        .filter(|dir| pos.can_move(&grid, *dir, width, height, player_id))
        .collect::<Vec<Dir>>();

    if dirs.len() == 0 {
        return Dir::None.into();
    }

    let idx = random::from_vec(&(0..dirs.len() as u32).collect::<Vec<u32>>()).unwrap();

    dirs[idx as usize].into()
}

fn pathfind_to_empty(
    grid: Vec<Cell>,
    player_id: u8,
    pos: (u32, u32),
    _brain: Brain,
    width: u32,
    height: u32,
) -> Action {
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

export!(PathfindPrioritiseEmptyStrategy);
