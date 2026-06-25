use std::{ffi::OsStr, fmt::Debug, fs, io, path::PathBuf, sync::LazyLock};

use wasmtime::{
    Engine, Store,
    component::{Component, HasSelf, Linker},
};

use crate::bindings::{Action, Brain, Cell, HostState, StrategyPlugin, game};

pub struct Strategy {
    name: String,
    store: Store<HostState>,
    instance: StrategyPlugin,
}

impl Strategy {
    pub fn new<T: Into<String>>(
        name: T,
        store: Store<HostState>,
        instance: StrategyPlugin,
    ) -> Self {
        Self {
            name: name.into(),
            store,
            instance,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn step(
        &mut self,
        grid: &[Cell],
        player_id: u8,
        pos: (usize, usize),
        brain: &Brain,
        width: usize,
        height: usize,
    ) -> Action {
        self.instance
            .call_step(
                &mut self.store,
                grid,
                player_id,
                (pos.0 as u32, pos.1 as u32),
                brain,
                width as u32,
                height as u32,
            )
            .unwrap()
    }
}

impl Debug for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Strategy").field(&self.name).finish()
    }
}

pub struct StrategyInfo {
    name: String,
    path: PathBuf,
}

impl StrategyInfo {
    pub fn build(&self) -> Strategy {
        load_strategy(&self.path).expect("Error while loading strategy")
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

static ENGINE: LazyLock<Engine> = LazyLock::new(|| Engine::default());

static LINKER: LazyLock<Linker<HostState>> = LazyLock::new(|| {
    let mut linker: Linker<HostState> = Linker::new(&ENGINE);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();

    game::strategy_plugin::logger::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)
        .unwrap();
    game::strategy_plugin::random::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)
        .unwrap();

    linker
});

pub static STRATEGIES: LazyLock<Vec<StrategyInfo>> = LazyLock::new(|| {
    let mut strategies = Vec::new();

    log::info!("Starting plugin loading");

    if let io::Result::Ok(dir) = fs::read_dir("strategies") {
        for entry in dir {
            if let Ok(file) = entry
                && file.path().is_file()
                && file.path().extension().and_then(OsStr::to_str) == Some("wasm")
            {
                log::info!("Loading plugin in file {:?}", file.file_name());

                match load_strategy(&file.path()) {
                    Ok(strategy) => {
                        log::info!(
                            "Successfully loaded strategy: {} from: {:?}",
                            &strategy.name,
                            file.file_name()
                        );

                        strategies.push(StrategyInfo {
                            name: strategy.name,
                            path: file.path(),
                        });
                    }
                    Err(err) => log::warn!(
                        "Error while loading strategy: {:?}: {}",
                        file.file_name(),
                        err
                    ),
                };
            }
        }
    }

    log::info!("All plugins loaded");

    strategies
});

fn load_strategy(path: &PathBuf) -> wasmtime::Result<Strategy> {
    let component = Component::from_file(&ENGINE, path)?;
    let mut store = Store::new(&ENGINE, HostState::default());
    let strategy = StrategyPlugin::instantiate(&mut store, &component, &LINKER)?;
    let strategy_name = strategy.call_get_name(&mut store)?;

    Ok(Strategy::new(strategy_name, store, strategy))
}
