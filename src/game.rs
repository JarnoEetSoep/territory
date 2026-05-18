use std::{collections::VecDeque, ops::Add};

use crate::{settings_panel::PlayerSettings, strategies::Strategies};

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

    pub fn step(&mut self) {
        let mut fill_enclosed_areas_players = Vec::new();

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

                let mut calculate_enclosed = false;

                match self.grid[new_pos.y * self.width + new_pos.x] {
                    Cell::Empty => {
                        calculate_enclosed = true;
                    }
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

                if calculate_enclosed {
                    fill_enclosed_areas_players.push(player.id);
                }
            }
        }

        for player_id in fill_enclosed_areas_players {
            self.fill_unreachable_areas(player_id);
        }
    }

    pub fn reset(&mut self, players: &[PlayerSettings]) {
        self.grid.clear();

        for _ in 0..self.width * self.height {
            self.grid.push(Cell::Empty);
        }

        for player in players {
            if player.enabled {
                self.move_player_to(player.x, player.y, player.id);
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
            strategy: Strategies::default(),
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

    fn fill_unreachable_areas(&mut self, player: u8) {
        let mut reachable = vec![false; self.grid.len()];
        let mut queue = VecDeque::new();

        for y in 0..self.height {
            for x in 0..self.width {
                match self.grid[y * self.width + x] {
                    Cell::Player(id) if id != player => {
                        reachable[y * self.width + x] = true;
                        queue.push_back((x, y));
                    }
                    _ => {}
                }
            }
        }

        while let Some((x, y)) = queue.pop_front() {
            let mut neighbours = Vec::new();

            if x > 0 {
                neighbours.push((x - 1, y));
            }

            if x < self.width - 1 {
                neighbours.push((x + 1, y));
            }

            if y > 0 {
                neighbours.push((x, y - 1));
            }

            if y < self.height - 1 {
                neighbours.push((x, y + 1));
            }

            for (nx, ny) in neighbours {
                if reachable[ny * self.width + nx] {
                    continue;
                }

                let traversable = match self.grid[ny * self.width + nx] {
                    Cell::Empty => true,
                    Cell::PlayerClaimed(id) if id != player => true,
                    _ => false,
                };

                if traversable {
                    reachable[ny * self.width + nx] = true;
                    queue.push_back((nx, ny));
                }
            }
        }

        for (idx, cell) in self.grid.iter_mut().enumerate() {
            if !reachable[idx] && matches!(cell, Cell::Empty) {
                *cell = Cell::PlayerClaimed(player);
            }
        }
    }
}
