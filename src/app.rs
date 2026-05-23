#[cfg(not(target_arch = "wasm32"))]
use core::time::Duration;
use std::collections::{HashMap, HashSet, VecDeque};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    sync::{
        Arc,
        atomic::Ordering,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use egui::{
    Align2, CentralPanel, Color32, ColorImage, MenuBar, Panel, Pos2, Rect, RichText, Sense,
    TextureHandle, Ui, Window, widgets,
};
#[cfg(not(target_arch = "wasm32"))]
use egui::{Key, Modifiers};
use egui_extras::{Column, TableBuilder};

use crate::{
    game::{Cell, Game},
    settings_panel::{Command, CorePlayerSettings, PlayerSettings, SettingsPanel},
};

#[cfg(not(target_arch = "wasm32"))]
pub enum AppMessage {
    Start,
    Stop,
    Terminate,
    SendCommand(Command),
}

#[derive(Debug)]
pub enum GameMessage {
    CellChanged(usize, usize, Cell),
    PlayerAdded(u8),
    PlayerRemoved(u8),
    PlayerMoved(u8, usize, usize),
    Pause,
}

pub struct RenderState {
    game_texture: TextureHandle,
    width: usize,
    height: usize,
    offset_x: f32,
    offset_y: f32,
    cell_size: f32,
    rect: Rect,
}

impl std::fmt::Debug for RenderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderState")
            .field("game_texture", &self.game_texture.name())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("offset_x", &self.offset_x)
            .field("offset_y", &self.offset_y)
            .field("cell_size", &self.cell_size)
            .field("rect", &self.rect)
            .finish()
    }
}

pub struct AppState {
    running: bool,
    settings_panel: SettingsPanel,
    #[cfg(target_arch = "wasm32")]
    game: Game,
    #[cfg(not(target_arch = "wasm32"))]
    game_thread: Option<JoinHandle<()>>,
    #[cfg(not(target_arch = "wasm32"))]
    tx: Sender<AppMessage>,
    #[cfg(not(target_arch = "wasm32"))]
    rx: Receiver<GameMessage>,
    claimed_amount: HashMap<u8, HashSet<(usize, usize)>>,
    #[cfg(target_arch = "wasm32")]
    last_step_time: Instant,
    changed_cells: VecDeque<(usize, usize, Cell)>,
    render_state: RenderState,
}

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let (game_tx, rx) = mpsc::channel();
        let game = Game::new(
            1,
            1,
            #[cfg(not(target_arch = "wasm32"))]
            game_tx,
        );
        #[cfg(not(target_arch = "wasm32"))]
        let mut game = game;
        let settings_panel = SettingsPanel::default();

        #[cfg(not(target_arch = "wasm32"))]
        let (tx, game_rx) = mpsc::channel::<AppMessage>();
        #[cfg(not(target_arch = "wasm32"))]
        let mut running = false;
        #[cfg(not(target_arch = "wasm32"))]
        let step_delay = Arc::clone(&settings_panel.step_delay);
        #[cfg(not(target_arch = "wasm32"))]
        let ctx = cc.egui_ctx.clone();

        #[cfg(not(target_arch = "wasm32"))]
        let handle = thread::spawn(move || {
            loop {
                match game_rx.try_recv() {
                    Ok(msg) => match msg {
                        AppMessage::Start => {
                            running = true;
                        }
                        AppMessage::Stop => {
                            running = false;
                        }
                        AppMessage::Terminate => break,
                        AppMessage::SendCommand(cmd) => game.run_command(cmd),
                    },
                    Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {
                        if running {
                            let delay = step_delay.load(Ordering::Relaxed);

                            thread::sleep(match delay {
                                0 => Duration::from_nanos(1),
                                _ => Duration::from_millis(delay),
                            });

                            game.step();

                            ctx.request_repaint();
                        }
                    }
                }
            }
        });

        Self {
            state: AppState {
                running: false,
                settings_panel,
                #[cfg(target_arch = "wasm32")]
                game,
                #[cfg(not(target_arch = "wasm32"))]
                game_thread: Some(handle),
                #[cfg(not(target_arch = "wasm32"))]
                tx,
                #[cfg(not(target_arch = "wasm32"))]
                rx,
                claimed_amount: HashMap::new(),
                #[cfg(target_arch = "wasm32")]
                last_step_time: Instant::now(),
                changed_cells: VecDeque::new(),
                render_state: RenderState {
                    game_texture: cc.egui_ctx.load_texture(
                        "game-image",
                        ColorImage::filled([1, 1], Color32::TRANSPARENT),
                        Default::default(),
                    ),
                    width: 1,
                    height: 1,
                    offset_x: 0.,
                    offset_y: 0.,
                    cell_size: 0.,
                    rect: Rect::ZERO,
                },
            },
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F11)) {
            use egui::ViewportCommand;

            let fullscreen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ui.send_viewport_cmd(ViewportCommand::Fullscreen(!fullscreen));
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ui.input(|i| i.viewport().close_requested()) {
            self.state
                .tx
                .send(AppMessage::Terminate)
                .expect("Error while sending Terminate ThreadMessage");
            self.state
                .game_thread
                .take()
                .expect("Error while take()-ing game thread")
                .join()
                .expect("Error while joining game thread");
        }

        let mut res = VecDeque::<GameMessage>::new();

        #[cfg(target_arch = "wasm32")]
        if self.state.running {
            if self.state.last_step_time.elapsed()
                > Duration::from_millis(self.state.settings_panel.step_delay)
            {
                self.state.game.step(&mut res);

                self.state.last_step_time = Instant::now();
            }

            ui.ctx().request_repaint();
        }

        let mut cmd = Command::Nothing;

        Panel::top("top_panel").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.visuals_mut().button_frame = false;

                self.bar_contents(ui, frame, &mut cmd);
            });
        });

        CentralPanel::default().show_inside(ui, |ui| {
            self.settings_panel(ui, frame, &mut cmd);

            self.game_panel(ui, frame, &cmd);

            self.stats_window(ui, frame);
        });

        #[cfg(not(target_arch = "wasm32"))]
        while let Ok(msg) = self.state.rx.try_recv() {
            res.push_back(msg);
        }

        if !matches!(cmd, Command::Nothing) {
            #[cfg(not(target_arch = "wasm32"))]
            self.state
                .tx
                .send(AppMessage::SendCommand(cmd))
                .expect("Error while sending SendCommand AppMessage");

            #[cfg(target_arch = "wasm32")]
            self.state.game.run_command(cmd, &mut res);
        }

        while let Some(msg) = res.pop_front() {
            self.handle_game_message(&msg);
        }
    }
}

impl App {
    fn settings_panel(&mut self, ui: &mut Ui, frame: &mut eframe::Frame, cmd: &mut Command) {
        let is_open =
            self.state.settings_panel.open || ui.memory(|mem| mem.everything_is_visible());

        Panel::left("settings_panel")
            .resizable(true)
            .show_animated_inside(ui, is_open, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Settings");
                });

                ui.separator();
                self.state.settings_panel.ui(ui, frame, cmd);
            });
    }

    fn bar_contents(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame, cmd: &mut Command) {
        ui.add_space(4.0);

        widgets::global_theme_preference_switch(ui);

        ui.separator();

        ui.toggle_value(&mut self.state.settings_panel.open, "Settings");

        ui.separator();

        if ui.button("Reset").clicked() {
            *cmd = Command::Reset(
                self.state
                    .settings_panel
                    .players_settings
                    .iter()
                    .map(|settings| settings.core_settings)
                    .collect(),
            );

            self.state
                .claimed_amount
                .iter_mut()
                .for_each(|(_, cells)| cells.clear());
        }

        ui.separator();

        let content = match (self.state.running, ui.visuals().dark_mode) {
            (true, true) => RichText::new("⏸").color(Color32::LIGHT_RED),
            (false, true) => RichText::new("▶").color(Color32::LIGHT_GREEN),
            (true, false) => RichText::new("⏸").color(Color32::RED),
            (false, false) => RichText::new("▶").color(Color32::GREEN),
        };

        if ui.button(content).clicked() {
            self.state.running = !self.state.running;

            #[cfg(not(target_arch = "wasm32"))]
            if self.state.running {
                self.state
                    .tx
                    .send(AppMessage::Start)
                    .expect("Error while sending Start AppMessage");
            } else {
                self.state
                    .tx
                    .send(AppMessage::Stop)
                    .expect("Error while sending Stop AppMessage");
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    fn game_panel(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame, cmd: &Command) {
        if ui.available_width() < 1. || ui.available_height() < 1. {
            return;
        }

        let (res, painter) = ui.allocate_painter(ui.available_size(), Sense::click());

        if let Command::ApplyGridSize(width, height) = cmd {
            self.state.render_state.width = *width;
            self.state.render_state.height = *height;
        }

        if res.rect.aspect_ratio() * self.state.render_state.height as f32
            > self.state.render_state.width as f32
        {
            self.state.render_state.cell_size =
                res.rect.height() / self.state.render_state.height as f32;

            self.state.render_state.offset_x = 0.5
                * (res.rect.width()
                    - self.state.render_state.cell_size * self.state.render_state.width as f32);
            self.state.render_state.offset_y = 0.;
        } else {
            self.state.render_state.cell_size =
                res.rect.width() / self.state.render_state.width as f32;

            self.state.render_state.offset_y = 0.5
                * (res.rect.height()
                    - self.state.render_state.cell_size * self.state.render_state.height as f32);
            self.state.render_state.offset_x = 0.;
        }

        let new_rect = Rect::from_two_pos(
            Pos2::new(
                res.rect.left() + self.state.render_state.offset_x,
                res.rect.top() + self.state.render_state.offset_y,
            ),
            Pos2::new(
                res.rect.right() - self.state.render_state.offset_x,
                res.rect.bottom() - self.state.render_state.offset_y,
            ),
        );

        if self.state.render_state.rect != new_rect {
            self.state.render_state.rect = new_rect;
            self.state.render_state.game_texture.set(
                ColorImage::filled(
                    [
                        self.state.render_state.width * self.state.render_state.cell_size as usize,
                        self.state.render_state.height * self.state.render_state.cell_size as usize,
                    ],
                    Color32::TRANSPARENT,
                ),
                Default::default(),
            );

            for y in 0..self.state.render_state.height {
                for x in 0..self.state.render_state.width {
                    self.draw_cell_at(x, y, Cell::Empty);
                }
            }

            let mut claimed = Vec::new();

            #[expect(clippy::iter_over_hash_type)]
            for (id, claimed_cells) in &self.state.claimed_amount {
                #[expect(clippy::iter_over_hash_type)]
                for (x, y) in claimed_cells {
                    claimed.push((*x, *y, *id));
                }
            }

            for (x, y, id) in claimed {
                self.draw_cell_at(x, y, Cell::PlayerClaimed(id));
            }

            let mut players = Vec::new();

            for player in &self.state.settings_panel.players_settings {
                if let Some((x, y)) = player.current_position {
                    players.push((x, y, player.core_settings.id));
                }
            }

            for (x, y, id) in players {
                self.draw_cell_at(x, y, Cell::Player(id));
            }
        }

        while let Some((x, y, cell)) = self.state.changed_cells.pop_front() {
            if let Cell::PlayerClaimed(id) = cell {
                match self.state.claimed_amount.get_mut(&id) {
                    Some(claimed) => {
                        claimed.insert((x, y));
                    }
                    None => {
                        self.state.claimed_amount.insert(id, HashSet::new());
                    }
                }
            }

            self.draw_cell_at(x, y, cell);
        }

        if let Command::ColorChanged(id) = cmd {
            let mut redraw_cells = Vec::new();

            if let Some(cells) = self.state.claimed_amount.get(id) {
                #[expect(clippy::iter_over_hash_type)]
                for (x, y) in cells {
                    redraw_cells.push((*x, *y));
                }
            }

            for (x, y) in redraw_cells {
                self.draw_cell_at(x, y, Cell::PlayerClaimed(*id));
            }

            if let Some((x, y)) = self
                .state
                .settings_panel
                .players_settings
                .iter()
                .find(|p| p.core_settings.id == *id)
                .expect("Player not found")
                .current_position
            {
                self.draw_cell_at(x, y, Cell::Player(*id));
            }
        }

        painter.image(
            self.state.render_state.game_texture.id(),
            self.state.render_state.rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    fn draw_cell_at(&mut self, x: usize, y: usize, cell: Cell) {
        let center = [
            x * self.state.render_state.cell_size as usize,
            y * self.state.render_state.cell_size as usize,
        ];

        let mut fill_color = match cell {
            Cell::Empty => Color32::GRAY.gamma_multiply(0.2),
            Cell::Player(id) | Cell::PlayerClaimed(id) => {
                let rgb = self
                    .state
                    .settings_panel
                    .players_settings
                    .iter()
                    .find(|settings| settings.core_settings.id == id)
                    .expect("Player not found")
                    .color;

                Color32::from_rgb(rgb[0], rgb[1], rgb[2])
            }
        };

        if let Cell::PlayerClaimed(_) = cell {
            fill_color = fill_color.gamma_multiply(0.5);
        }

        self.state.render_state.game_texture.set_partial(
            center,
            ColorImage::filled(
                [
                    self.state.render_state.cell_size as usize,
                    self.state.render_state.cell_size as usize,
                ],
                fill_color,
            ),
            Default::default(),
        );
    }

    fn stats_window(&self, ui: &Ui, _frame: &mut eframe::Frame) {
        Window::new("Stats")
            .resizable(false)
            .pivot(Align2::LEFT_BOTTOM)
            .default_pos([0., ui.ctx().viewport_rect().height()])
            .show(ui.ctx(), |ui| {
                TableBuilder::new(ui)
                    .id_salt("stats")
                    .striped(true)
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .body(|mut body| {
                        let total =
                            self.state.settings_panel.width * self.state.settings_panel.height;

                        let mut values =
                            self.state
                                .claimed_amount
                                .iter()
                                .collect::<Vec<(&u8, &HashSet<(usize, usize)>)>>();

                        values.sort_by_key(|val| val.1.len());

                        for &(player_id, cells_taken) in values.iter().rev() {
                            let settings = self
                                .state
                                .settings_panel
                                .players_settings
                                .iter()
                                .find(|player| player.core_settings.id == *player_id)
                                .expect("Player not found");

                            if settings.core_settings.enabled {
                                body.row(20.0, |mut row| {
                                    row.col(|ui| {
                                        let (r, g, b) = (
                                            settings.color[0],
                                            settings.color[1],
                                            settings.color[2],
                                        );
                                        ui.colored_label(Color32::from_rgb(r, g, b), "⬛");
                                    });

                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}:", settings.name));
                                        });
                                    });

                                    let mut taken = cells_taken.len() + 1;

                                    if let Some((x, y)) = settings.current_position
                                        && cells_taken.contains(&(x, y))
                                    {
                                        taken -= 1;
                                    }

                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "{:.2}%",
                                                taken as f32 / total as f32 * 100.
                                            ));
                                        });
                                    });
                                });
                            }
                        }
                    });
            });
    }

    fn handle_game_message(&mut self, msg: &GameMessage) {
        match *msg {
            GameMessage::CellChanged(x, y, cell) => {
                self.state.changed_cells.push_back((x, y, cell));
            }
            GameMessage::PlayerAdded(id) => {
                self.state
                    .settings_panel
                    .players_settings
                    .push(PlayerSettings {
                        core_settings: CorePlayerSettings {
                            id,
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                self.state.claimed_amount.insert(id, HashSet::new());
            }
            GameMessage::PlayerRemoved(id) => {
                self.state
                    .settings_panel
                    .players_settings
                    .retain(|player| player.core_settings.id != id);

                self.state
                    .claimed_amount
                    .retain(|&player_id, _| player_id != id);

                self.state.claimed_amount.remove(&id);
            }
            GameMessage::PlayerMoved(id, x, y) => {
                let player = self
                    .state
                    .settings_panel
                    .players_settings
                    .iter_mut()
                    .find(|player| player.core_settings.id == id)
                    .expect("Player not found");

                player.current_position = Some((x, y));
            }
            GameMessage::Pause => {
                self.state.running = false;

                #[cfg(not(target_arch = "wasm32"))]
                self.state
                    .tx
                    .send(AppMessage::Stop)
                    .expect("Error while sending Stop AppMessage");
            }
        }
    }
}
