use std::ops::Add;

use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

wasmtime::component::bindgen!("strategy-plugin");

pub struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl HostState {
    pub fn new(wasi: WasiCtx, table: ResourceTable) -> Self {
        Self { wasi, table }
    }
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            wasi: WasiCtx::builder().build(),
            table: ResourceTable::new(),
        }
    }
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl game::strategy_plugin::logger::Host for HostState {
    fn error(&mut self, msg: String) {
        log::error!("{}", msg);
    }

    fn warn(&mut self, msg: String) {
        log::warn!("{}", msg);
    }

    fn info(&mut self, msg: String) {
        log::info!("{}", msg);
    }

    fn debug(&mut self, msg: String) {
        log::debug!("{}", msg);
    }
}

impl game::strategy_plugin::random::Host for HostState {
    fn from_vec(&mut self, range: Vec<u32>) -> Option<u32> {
        fastrand::choice(range)
    }
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

    pub fn from_to(from: (usize, usize), to: (usize, usize)) -> Self {
        let dx: isize = to.0.cast_signed() - from.0.cast_signed();
        let dy: isize = to.1.cast_signed() - from.1.cast_signed();

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

impl Add<Dir> for (usize, usize) {
    type Output = Self;

    fn add(self, rhs: Dir) -> Self::Output {
        match rhs {
            Dir::None => self,
            Dir::Up => (self.0, self.1 - 1),
            Dir::Down => (self.0, self.1 + 1),
            Dir::Left => (self.0 - 1, self.1),
            Dir::Right => (self.0 + 1, self.1),
        }
    }
}

impl Default for Dir {
    fn default() -> Self {
        Self::None
    }
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
            facing: Default::default(),
            memory: Default::default(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty
    }
}
