#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::Sender;
use std::{collections::VecDeque, ops::Add, sync::Arc};

use crate::app::GameMessage;
use crate::{
    settings_panel::{Command, CorePlayerSettings},
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

impl From<Pos> for (usize, usize) {
    fn from(value: Pos) -> Self {
        (value.x, value.y)
    }
}

impl Pos {
    pub const ZERO: Self = Self { x: 0, y: 0 };

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

    pub fn can_move(&self, grid: &[Cell], dir: Dir, width: usize, height: usize, id: u8) -> bool {
        if self.x == 0 && matches!(dir, Dir::Left)
            || self.x == width - 1 && matches!(dir, Dir::Right)
            || self.y == 0 && matches!(dir, Dir::Up)
            || self.y == height - 1 && matches!(dir, Dir::Down)
        {
            return false;
        }

        let new_pos = *self + dir;

        match grid[new_pos.y * width + new_pos.x] {
            Cell::Empty => true,
            Cell::Player(player_id) | Cell::PlayerClaimed(player_id) if player_id == id => true,
            _ => false,
        }
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

#[derive(Debug, Clone)]
pub struct Brain {
    pub strategy: Arc<dyn Strategy>,
    pub facing: Dir,
    pub memory: Vec<u8>,
}

impl Brain {
    pub fn reset(&mut self) {
        self.facing = Dir::None;
        self.memory.clear();
    }
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

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u8,
    pub position: Option<Pos>,
    pub brain: Brain,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum Cell {
    #[default]
    Empty,
    Player(u8),
    PlayerClaimed(u8),
}

pub struct Game {
    pub players: Vec<Player>,
    grid: Vec<Cell>,
    pub width: usize,
    pub height: usize,
    last_id: u8,
    #[cfg(not(target_arch = "wasm32"))]
    tx: Sender<GameMessage>,
}

impl Game {
    pub fn new(
        width: usize,
        height: usize,
        #[cfg(not(target_arch = "wasm32"))] sender: Sender<GameMessage>,
    ) -> Self {
        Self {
            players: Vec::new(),
            grid: vec![Cell::default(); width * height],
            width,
            height,
            last_id: 0,
            #[cfg(not(target_arch = "wasm32"))]
            tx: sender,
        }
    }

    pub fn step(&mut self, #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>) {
        let mut fill_enclosed_areas_players = Vec::new();
        let mut updated_positions = Vec::new();

        self.players.iter_mut().for_each(|player| {
            if let Some(pos) = player.position {
                let dir = Arc::clone(&player.brain.strategy).step(
                    &self.grid,
                    player.id,
                    player.position.expect("Player has no position"),
                    &mut player.brain,
                    self.width,
                    self.height,
                );

                if !pos.can_move(&self.grid, dir, self.width, self.height, player.id) {
                    return;
                }

                let new_pos = pos + dir;

                if matches!(self.grid[new_pos.y * self.width + new_pos.x], Cell::Empty) {
                    fill_enclosed_areas_players.push(player.id);
                }

                updated_positions.push((new_pos.x, new_pos.y, player.id));

                self.grid[pos.y * self.width + pos.x] = Cell::PlayerClaimed(player.id);

                let res = GameMessage::CellChanged(pos.x, pos.y, Cell::PlayerClaimed(player.id));

                #[cfg(not(target_arch = "wasm32"))]
                self.tx
                    .send(res)
                    .expect("Error while sending CellChanged GameMessage");

                #[cfg(target_arch = "wasm32")]
                response.push_back(res);

                self.grid[new_pos.y * self.width + new_pos.x] = Cell::Player(player.id);

                let res = GameMessage::CellChanged(new_pos.x, new_pos.y, Cell::Player(player.id));

                #[cfg(not(target_arch = "wasm32"))]
                self.tx
                    .send(res)
                    .expect("Error while sending CellChanged GameMessage");

                #[cfg(target_arch = "wasm32")]
                response.push_back(res);

                player.position = Some(new_pos);

                let res = GameMessage::PlayerMoved(player.id, new_pos.x, new_pos.y);

                #[cfg(not(target_arch = "wasm32"))]
                self.tx
                    .send(res)
                    .expect("Error while sending PlayerMoved GameMessage");

                #[cfg(target_arch = "wasm32")]
                response.push_back(res);
            }
        });

        for player_id in fill_enclosed_areas_players {
            self.fill_unreachable_areas(
                player_id,
                #[cfg(target_arch = "wasm32")]
                response,
            );
        }

        if self
            .grid
            .iter()
            .filter(|cell| matches!(cell, Cell::Empty))
            .count()
            == 0
        {
            let res = GameMessage::Pause;

            #[cfg(not(target_arch = "wasm32"))]
            self.tx
                .send(res)
                .expect("Error while sending Pause GameMessage");

            #[cfg(target_arch = "wasm32")]
            response.push_back(res);
        }
    }

    pub fn reset(
        &mut self,
        players: &[CorePlayerSettings],
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_cell_at(
                    x,
                    y,
                    Cell::Empty,
                    #[cfg(target_arch = "wasm32")]
                    response,
                );
            }
        }

        self.players
            .iter_mut()
            .for_each(|player| player.brain.reset());

        for player in players {
            if player.enabled {
                self.move_player_to(
                    player.x,
                    player.y,
                    player.id,
                    Cell::Empty,
                    #[cfg(target_arch = "wasm32")]
                    response,
                );
            }
        }
    }

    pub fn set_player_strategy(&mut self, id: u8, strategy: &Arc<dyn Strategy>) {
        self.players
            .iter_mut()
            .find(|player| player.id == id)
            .expect("Player not found")
            .brain
            .strategy = Arc::clone(strategy);
    }

    pub fn move_player_to(
        &mut self,
        x: usize,
        y: usize,
        id: u8,
        leave_behind: Cell,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
        if let Some(pos) = self
            .players
            .iter_mut()
            .find(|player| player.id == id)
            .expect("Player not found")
            .position
            .replace(Pos { x, y })
        {
            self.set_cell_at(
                pos.x,
                pos.y,
                leave_behind,
                #[cfg(target_arch = "wasm32")]
                response,
            );
        }

        self.set_cell_at(
            x,
            y,
            Cell::Player(id),
            #[cfg(target_arch = "wasm32")]
            response,
        );

        let res = GameMessage::PlayerMoved(id, x, y);

        #[cfg(not(target_arch = "wasm32"))]
        self.tx
            .send(res)
            .expect("Error while sending PlayerMoved GameMessage");

        #[cfg(target_arch = "wasm32")]
        response.push_back(res);
    }

    pub fn disable_player(
        &mut self,
        id: u8,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
        if let Some(pos) = self
            .players
            .iter_mut()
            .find(|player| player.id == id)
            .expect("Player not found")
            .position
            .take()
        {
            self.set_cell_at(
                pos.x,
                pos.y,
                Cell::Empty,
                #[cfg(target_arch = "wasm32")]
                response,
            );
        }
    }

    pub fn resize(
        &mut self,
        width: usize,
        height: usize,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
        self.width = width;
        self.height = height;
        self.grid = vec![Cell::default(); width * height];

        for player in self.players.clone() {
            self.disable_player(
                player.id,
                #[cfg(target_arch = "wasm32")]
                response,
            );
        }
    }

    pub fn get_cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.grid[y * self.width + x]
    }

    pub fn set_cell_at(
        &mut self,
        x: usize,
        y: usize,
        cell: Cell,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
        self.grid[y * self.width + x] = cell;

        let res = GameMessage::CellChanged(x, y, cell);

        #[cfg(not(target_arch = "wasm32"))]
        self.tx
            .send(res)
            .expect("Error while sending CellChanged GameMessage");

        #[cfg(target_arch = "wasm32")]
        response.push_back(res);
    }

    pub fn add_player(
        &mut self,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) -> u8 {
        self.last_id += 1;

        self.players.push(Player {
            id: self.last_id,
            position: None,
            brain: Brain::default(),
        });

        let res = GameMessage::PlayerAdded(self.last_id);

        #[cfg(not(target_arch = "wasm32"))]
        self.tx
            .send(res)
            .expect("Error while sending PlayerAdded GameMessage");

        #[cfg(target_arch = "wasm32")]
        response.push_back(res);

        self.last_id
    }

    pub fn remove_player(
        &mut self,
        id: u8,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
        self.players.retain(|player| player.id != id);

        for y in 0..self.height {
            for x in 0..self.width {
                match self.get_cell_at(x, y) {
                    Cell::Player(player_id) | Cell::PlayerClaimed(player_id)
                        if *player_id == id =>
                    {
                        self.set_cell_at(
                            x,
                            y,
                            Cell::Empty,
                            #[cfg(target_arch = "wasm32")]
                            response,
                        );
                    }
                    _ => {}
                }
            }
        }

        let res = GameMessage::PlayerRemoved(id);

        #[cfg(not(target_arch = "wasm32"))]
        self.tx
            .send(res)
            .expect("Error while sending PlayerRemoved GameMessage");

        #[cfg(target_arch = "wasm32")]
        response.push_back(res);
    }

    fn fill_unreachable_areas(
        &mut self,
        player: u8,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
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

        for y in 0..self.height {
            for x in 0..self.width {
                if !reachable[y * self.width + x] && matches!(self.get_cell_at(x, y), Cell::Empty) {
                    self.set_cell_at(
                        x,
                        y,
                        Cell::PlayerClaimed(player),
                        #[cfg(target_arch = "wasm32")]
                        response,
                    );
                }
            }
        }
    }

    pub fn run_command(
        &mut self,
        cmd: Command,
        #[cfg(target_arch = "wasm32")] response: &mut VecDeque<GameMessage>,
    ) {
        match cmd {
            Command::ApplyGridSize(width, height) => self.resize(
                width,
                height,
                #[cfg(target_arch = "wasm32")]
                response,
            ),
            Command::AddPlayer => {
                self.add_player(
                    #[cfg(target_arch = "wasm32")]
                    response,
                );
            }
            Command::RemovePlayer(id) => self.remove_player(
                id,
                #[cfg(target_arch = "wasm32")]
                response,
            ),
            Command::SetStrategy(id, strategy) => {
                self.set_player_strategy(
                    id,
                    STRATEGIES.get(&strategy).expect("Strategy not found"),
                );
            }
            Command::MovePlayer(id, x, y) => self.move_player_to(
                x,
                y,
                id,
                Cell::Empty,
                #[cfg(target_arch = "wasm32")]
                response,
            ),
            Command::DisablePlayer(id) => self.disable_player(
                id,
                #[cfg(target_arch = "wasm32")]
                response,
            ),
            Command::Reset(players) => self.reset(
                &players,
                #[cfg(target_arch = "wasm32")]
                response,
            ),
            _ => {}
        }
    }
}
