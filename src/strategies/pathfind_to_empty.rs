use std::collections::{HashMap, VecDeque};

use crate::{
    game::{Cell, Dir, Player},
    strategies::Strategy,
};

pub struct PathfindToEmptyStrategy;

impl Strategy for PathfindToEmptyStrategy {
    fn get_name(&self) -> &'static str {
        "Pathfind to empty"
    }

    fn step(&self, grid: &[Cell], player: &mut Player, width: usize, height: usize) -> Dir {
        let pos = player.position.expect("Player doesn't have a position");

        let mut target = None;

        let mut queue = VecDeque::new();
        let mut visited = vec![false; grid.len()];
        let mut parent = HashMap::new();

        queue.push_back((pos.x, pos.y));
        visited[pos.y * width + pos.x] = true;

        while let Some((x, y)) = queue.pop_front()
            && target.is_none()
        {
            let mut neighbours = Vec::new();

            if x > 0 {
                neighbours.push((x - 1, y));
            }

            if x < width - 1 {
                neighbours.push((x + 1, y));
            }

            if y > 0 {
                neighbours.push((x, y - 1));
            }

            if y < height - 1 {
                neighbours.push((x, y + 1));
            }

            for (nx, ny) in neighbours {
                if visited[ny * width + nx] {
                    continue;
                }

                parent.insert((nx, ny), (x, y));

                match grid[ny * width + nx] {
                    Cell::Empty => target = Some((nx, ny)),
                    Cell::PlayerClaimed(id) if id == player.id => {
                        visited[ny * width + nx] = true;

                        queue.push_back((nx, ny));
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

            return Dir::from_to(path[0].into(), path[1].into());
        }

        Dir::None
    }
}
