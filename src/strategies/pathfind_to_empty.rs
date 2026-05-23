use std::collections::{HashMap, VecDeque};

use crate::{
    game::{Brain, Cell, Dir, Pos},
    strategies::Strategy,
};

pub struct PathfindToEmptyStrategy;

impl Strategy for PathfindToEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Pathfind to empty"
    }

    fn step(
        &self,
        grid: &[Cell],
        player_id: u8,
        pos: Pos,
        _brain: &mut Brain,
        width: usize,
        height: usize,
    ) -> Dir {
        let mut target = None;

        let mut queue = VecDeque::new();
        let mut visited = vec![false; grid.len()];
        let mut parent = HashMap::new();

        queue.push_back(pos);
        visited[pos.y * width + pos.x] = true;

        while let Some(pos) = queue.pop_front()
            && target.is_none()
        {
            for neighbour in pos.neighbours(width, height) {
                if visited[neighbour.y * width + neighbour.x] {
                    continue;
                }

                parent.insert(neighbour, pos);

                match grid[neighbour.y * width + neighbour.x] {
                    Cell::Empty => target = Some(neighbour),
                    Cell::PlayerClaimed(id) if id == player_id => {
                        visited[neighbour.y * width + neighbour.x] = true;

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

            return Dir::from_to(path[0], path[1]);
        }

        Dir::None
    }
}
