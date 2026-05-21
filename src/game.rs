use std::{collections::VecDeque, ops::Add, sync::Arc};

use crate::{
    settings_panel::PlayerSettings,
    strategies::{STRATEGIES, Strategy},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Pos {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl Pos {
    pub fn neighbours(self, width: usize, height: usize) -> Vec<Self> {
        let mut neighbours_positions = Vec::new();

        if self.x > 0 {
            neighbours_positions.push(self + Dir::Left);
        }

        if self.x < width - 1 {
            neighbours_positions.push(self + Dir::Right);
        }

        if self.y > 0 {
            neighbours_positions.push(self + Dir::Up);
        }

        if self.y < height - 1 {
            neighbours_positions.push(self + Dir::Down);
        }

        neighbours_positions
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Dir {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn right(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Up => Self::Right,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Right => Self::Down,
        }
    }

    pub fn left(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Up => Self::Left,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Right => Self::Up,
        }
    }

    pub fn from_to(from: Pos, to: Pos) -> Self {
        let dx: isize = to.x.cast_signed() - from.x.cast_signed();
        let dy: isize = to.y.cast_signed() - from.y.cast_signed();

        match (dx, dy) {
            (0, 0) => Self::None,
            (1, 0) => Self::Right,
            (-1, 0) => Self::Left,
            (0, 1) => Self::Down,
            (0, -1) => Self::Up,
            _ => panic!("From and to are not adjacent"),
        }
    }
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
pub struct Brain {
    pub strategy: Arc<dyn Strategy>,
    pub facing: Dir,
    pub memory: Vec<u8>,
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            strategy: Arc::clone(STRATEGIES.get(&0).expect("Strategy not found")),
            facing: Default::default(),
            memory: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub id: u8,
    pub position: Option<Pos>,
    pub brain: Brain,
}

impl Player {
    pub fn can_move(&self, grid: &[Cell], dir: Dir, width: usize, height: usize) -> bool {
        match self.position {
            Some(pos) => {
                if pos.x == 0 && matches!(dir, Dir::Left)
                    || pos.x == width - 1 && matches!(dir, Dir::Right)
                    || pos.y == 0 && matches!(dir, Dir::Up)
                    || pos.y == height - 1 && matches!(dir, Dir::Down)
                {
                    return false;
                }

                let new_pos = pos + dir;

                match grid[new_pos.y * width + new_pos.x] {
                    Cell::Empty => true,
                    Cell::Player(player_id) | Cell::PlayerClaimed(player_id)
                        if player_id == self.id =>
                    {
                        true
                    }
                    _ => false,
                }
            }
            None => false,
        }
    }
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
                let dir = Arc::clone(&player.brain.strategy).step(
                    &self.grid,
                    player,
                    self.width,
                    self.height,
                );

                if !player.can_move(&self.grid, dir, self.width, self.height) {
                    return;
                }

                let new_pos = pos + dir;

                if matches!(self.grid[new_pos.y * self.width + new_pos.x], Cell::Empty) {
                    fill_enclosed_areas_players.push(player.id);
                }

                self.grid[pos.y * self.width + pos.x] = Cell::PlayerClaimed(player.id);
                self.grid[new_pos.y * self.width + new_pos.x] = Cell::Player(player.id);
                player.position = Some(new_pos);
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

        for player in &mut self.players {
            player.brain = Brain {
                strategy: Arc::clone(&player.brain.strategy),
                ..Default::default()
            };

            if let Some(pos) = player.position {
                let settings = players
                    .iter()
                    .find(|p| p.id == player.id)
                    .expect("No settings found for player");

                self.grid[pos.y * self.width + pos.x] = Cell::Empty;

                player.position = Some(Pos {
                    x: settings.x,
                    y: settings.y,
                });
                self.grid[settings.y * self.width + settings.x] = Cell::Player(player.id);
            }
        }

        for player in players {
            if player.enabled {
                self.move_player_to(player.x, player.y, player.id);
            }
        }
    }

    pub fn set_player_strategy(&mut self, id: u8, strategy: &Arc<dyn Strategy>) {
        for player in &mut self.players {
            if player.id == id {
                player.brain.strategy = Arc::clone(strategy);
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
            position: None,
            brain: Brain::default(),
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
