use std::{collections::VecDeque, ops::Add};

use crate::strategies::Strategies;

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Dir {
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Add<Dir> for Pos {
    type Output = Self;

    fn add(self, rhs: Dir) -> Self::Output {
        match rhs {
            Dir::None => self,
            Dir::Up => Self {
                x: self.x,
                y: self.y - 1,
            },
            Dir::Down => Self {
                x: self.x,
                y: self.y + 1,
            },
            Dir::Left => Self {
                x: self.x - 1,
                y: self.y,
            },
            Dir::Right => Self {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

impl Pos {
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Debug)]
pub struct Player {
    pub id: u8,
    pub strategy: Strategies,
    pub position: Option<Pos>,
}

#[derive(Default, Clone, Copy)]
pub enum Cell {
    #[default]
    Empty,
    Player(u8),
    PlayerClaimed(u8),
}

#[derive(Default)]
pub struct Game {
    pub players: Vec<Player>,
    grid: Vec<Cell>,
    pub width: usize,
    pub height: usize,
    last_id: u8,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![Cell::default(); width * height],
            ..Default::default()
        }
    }

    pub fn with_players(mut self, players: Vec<Player>) -> Self {
        self.players = players;

        self
    }

    pub fn step(&mut self) {
        for player in &mut self.players {
            if let Some(pos) = player.position {
                let dir =
                    player
                        .strategy
                        .get()
                        .step(&self.grid, self.width, self.height, pos, player.id);
                let new_pos = pos + dir;

                assert!(
                    new_pos.x < self.width && new_pos.y < self.height,
                    "Player with strategy {} moved out of bounds",
                    player.strategy.get().get_name()
                );

                match self.grid[new_pos.y * self.width + new_pos.x] {
                    Cell::Empty => {}
                    Cell::Player(id) => {
                        assert!(
                            id == player.id,
                            "Player with strategy {} tried to move on top of other player",
                            player.strategy.get().get_name()
                        );
                    }
                    Cell::PlayerClaimed(id) => {
                        assert!(
                            id == player.id,
                            "Player with strategy {} tried to move on territory of other player",
                            player.strategy.get().get_name()
                        );
                    }
                }

                self.grid[pos.y * self.width + pos.x] = Cell::PlayerClaimed(player.id);
                self.grid[new_pos.y * self.width + new_pos.x] = Cell::Player(player.id);
                player.position = Some(new_pos);

                capture_enclosed(&mut self.grid, self.width, self.height, player.id);
            }
        }
    }

    pub fn set_player_strategy(&mut self, id: u8, strategy: Strategies) {
        for player in &mut self.players {
            if player.id == id {
                player.strategy = strategy;
            }
        }
    }

    pub fn move_player_to(&mut self, x: usize, y: usize, id: u8) {
        for player in &mut self.players {
            if player.id == id {
                if let Some(pos) = player.position {
                    self.grid[pos.y * self.width + pos.x] = Cell::Empty;
                }

                player.position = Some(Pos { x, y });
                self.grid[y * self.width + x] = Cell::Player(player.id);
            }
        }
    }

    pub fn disable_player(&mut self, id: u8) {
        for player in &mut self.players {
            if player.id == id {
                if let Some(pos) = player.position {
                    self.grid[pos.y * self.width + pos.x] = Cell::PlayerClaimed(player.id);
                }

                player.position = None;
            }
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.grid = vec![Cell::default(); width * height];
    }

    pub fn get_cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.grid[y * self.width + x]
    }

    pub fn set_cell_at(&mut self, x: usize, y: usize, cell: Cell) {
        self.grid[y * self.width + x] = cell;
    }

    pub fn add_player(&mut self) -> u8 {
        self.last_id += 1;

        self.players.push(Player {
            id: self.last_id,
            strategy: Strategies::RandomWalkStrategy,
            position: None,
        });

        self.last_id
    }

    pub fn remove_player(&mut self, id: u8) {
        self.players.retain(|player| player.id != id);

        for cell in &mut self.grid {
            match cell {
                Cell::Empty => {}
                Cell::Player(player_id) | Cell::PlayerClaimed(player_id) => {
                    if *player_id == id {
                        *cell = Cell::Empty;
                    }
                }
            }
        }
    }
}

fn capture_enclosed(grid: &mut [Cell], width: usize, height: usize, player_id: u8) {
    let mut visited = vec![false; width * height];
    let mut queue = VecDeque::new();

    let is_wall = |cell: Cell, _x: usize, _y: usize| {
        matches!(
            cell,
            Cell::Player(id) | Cell::PlayerClaimed(id)
                if id == player_id
        )
    };

    for x in 0..width {
        enqueue_if_open(grid, &mut visited, &mut queue, x, 0, width, &is_wall);
        enqueue_if_open(
            grid,
            &mut visited,
            &mut queue,
            x,
            height - 1,
            width,
            &is_wall,
        );
    }

    for y in 0..height {
        enqueue_if_open(grid, &mut visited, &mut queue, 0, y, width, &is_wall);
        enqueue_if_open(
            grid,
            &mut visited,
            &mut queue,
            width - 1,
            y,
            width,
            &is_wall,
        );
    }

    // BFS flood-fill
    while let Some((x, y)) = queue.pop_front() {
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            if nx < width && ny < height {
                enqueue_if_open(grid, &mut visited, &mut queue, nx, ny, width, &is_wall);
            }
        }
    }

    // Any unvisited non-wall cell is enclosed
    for y in 0..height {
        for x in 0..width {
            if !visited[y * width + x] && !is_wall(grid[y * width + x], x, y) {
                grid[y * width + x] = Cell::PlayerClaimed(player_id);
            }
        }
    }
}

fn enqueue_if_open<F>(
    grid: &[Cell],
    visited: &mut [bool],
    queue: &mut VecDeque<(usize, usize)>,
    x: usize,
    y: usize,
    width: usize,
    is_wall: &F,
) where
    F: Fn(Cell, usize, usize) -> bool,
{
    if !visited[y * width + x] && !is_wall(grid[y * width + x], x, y) {
        visited[y * width + x] = true;
        queue.push_back((x, y));
    }
}
