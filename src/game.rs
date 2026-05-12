use std::{collections::VecDeque, ops::Add};

use crate::strategies::Strategies;

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: u16,
    pub y: u16
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Dir {
    None,
    Up,
    Down,
    Left,
    Right
}

impl Add<Dir> for Pos {
    type Output = Pos;

    fn add(self, rhs: Dir) -> Self::Output {
        match rhs {
            Dir::None => self,
            Dir::Up => Pos { x: self.x, y: self.y - 1 },
            Dir::Down => Pos { x: self.x, y: self.y + 1 },
            Dir::Left => Pos { x: self.x - 1, y: self.y },
            Dir::Right => Pos { x: self.x + 1, y: self.y },
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
    pub position: Option<Pos>
}

#[derive(Default, Clone, Copy)]
pub enum Cell {
    #[default]
    Empty,
    Player(u8),
    PlayerClaimed(u8)
}

#[derive(Default)]
pub struct Game {
    pub players: Vec<Player>,
    grid: Vec<Vec<Cell>>,
    pub width: u16,
    pub height: u16,
    last_id: u8
}

impl Game {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            grid: vec![vec![Cell::default(); usize::from(width)]; usize::from(height)],
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
                let dir = player.strategy.get().step(&self.grid, pos, player.id);
                let new_pos = pos + dir;

                if new_pos.x > self.width - 1 || new_pos.y > self.height - 1 {
                    panic!("Player with strategy {} moved out of bounds", player.strategy.get().get_name());
                }

                match self.grid[usize::from(new_pos.y)][usize::from(new_pos.x)] {
                    Cell::Empty => {},
                    Cell::Player(id) => {
                        if id != player.id {
                            panic!("Player with strategy {} tried to move on top of other player", player.strategy.get().get_name());
                        }
                    },
                    Cell::PlayerClaimed(id) => {
                        if id != player.id {
                            panic!("Player with strategy {} tried to move on territory of other player", player.strategy.get().get_name());
                        }
                    },
                }
                
                self.grid[usize::from(pos.y)][usize::from(pos.x)] = Cell::PlayerClaimed(player.id);
                self.grid[usize::from(new_pos.y)][usize::from(new_pos.x)] = Cell::Player(player.id);
                player.position = Some(new_pos);

                capture_enclosed(&mut self.grid, player.id);
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

    pub fn move_player_to(&mut self, x: u16, y: u16, id: u8) {
        for player in &mut self.players {
            if player.id == id {
                if let Some(pos) = player.position {
                    self.grid[usize::from(pos.y)][usize::from(pos.x)] = Cell::Empty;
                }

                player.position = Some(Pos { x, y });
                self.grid[usize::from(y)][usize::from(x)] = Cell::Player(player.id);
            }
        }
    }

    pub fn disable_player(&mut self, id: u8) {
        for player in &mut self.players {
            if player.id == id {
                if let Some(pos) = player.position {
                    self.grid[usize::from(pos.y)][usize::from(pos.x)] = Cell::PlayerClaimed(player.id);
                }

                player.position = None;
            }
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.grid = vec![vec![Cell::default(); usize::from(width)]; usize::from(height)];
    }

    pub fn get_cell_at(&self, x: u16, y: u16) -> &Cell {
        &self.grid[usize::from(y)][usize::from(x)]
    }

    pub fn set_cell_at(&mut self, x: u16, y: u16, cell: Cell) {
        self.grid[usize::from(y)][usize::from(x)] = cell;
    }

    pub fn add_player(&mut self) -> u8 {
        self.last_id += 1;

        self.players.push(Player {
            id: self.last_id,
            strategy: Strategies::RandomWalkStrategy,
            position: None
        });

        self.last_id
    }

    pub fn remove_player(&mut self, id: u8) {
        self.players.retain(|player| player.id != id);

        for row in &mut self.grid {
            for cell in row {
                match cell {
                    Cell::Empty => {},
                    Cell::Player(player_id) | Cell::PlayerClaimed(player_id) => {
                        if *player_id == id {
                            *cell = Cell::Empty;
                        }
                    }
                }
            }
        }
    }
}

fn capture_enclosed(grid: &mut Vec<Vec<Cell>>, player_id: u8) {
    let height = grid.len();
    let width = grid[0].len();

    let mut visited = vec![vec![false; width]; height];
    let mut queue = VecDeque::new();

    // Returns true if the cell blocks flood-fill
    let is_wall = |cell: Cell, _x: usize, _y: usize| {
        matches!(
            cell,
            Cell::Player(id) | Cell::PlayerClaimed(id)
                if id == player_id
        )
    };

    // Add border cells to queue
    for x in 0..width {
        enqueue_if_open(grid, &mut visited, &mut queue, x, 0, &is_wall);
        enqueue_if_open(grid, &mut visited, &mut queue, x, height - 1, &is_wall);
    }

    for y in 0..height {
        enqueue_if_open(grid, &mut visited, &mut queue, 0, y, &is_wall);
        enqueue_if_open(grid, &mut visited, &mut queue, width - 1, y, &is_wall);
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
                enqueue_if_open(
                    grid,
                    &mut visited,
                    &mut queue,
                    nx,
                    ny,
                    &is_wall,
                );
            }
        }
    }
    
    // Any unvisited non-wall cell is enclosed
    for y in 0..height {
        for x in 0..width {
            if !visited[y][x] && !is_wall(grid[y][x], x, y) {
                grid[y][x] = Cell::PlayerClaimed(player_id);
            }
        }
    }
}

fn enqueue_if_open<F>(
    grid: &Vec<Vec<Cell>>,
    visited: &mut Vec<Vec<bool>>,
    queue: &mut VecDeque<(usize, usize)>,
    x: usize,
    y: usize,
    is_wall: &F,
)
where
    F: Fn(Cell, usize, usize) -> bool,
{
    if !visited[y][x] && !is_wall(grid[y][x], x, y) {
        visited[y][x] = true;
        queue.push_back((x, y));
    }
}