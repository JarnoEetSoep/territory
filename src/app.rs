#[cfg(not(target_arch = "wasm32"))]
use core::time;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    sync::{
        atomic::Ordering,
        mpsc::{self, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use egui::{
    Align2, CentralPanel, Color32, MenuBar, Panel, Pos2, Rect, RichText, Sense, Stroke, StrokeKind,
    Ui, Vec2, Window, widgets,
};
#[cfg(not(target_arch = "wasm32"))]
use egui::{Key, Modifiers};
use egui_extras::{Column, TableBuilder};

use crate::{
    game::{Cell, Game},
    settings_panel::{Command, SettingsPanel},
    strategies::STRATEGIES,
};

#[cfg(not(target_arch = "wasm32"))]
pub enum ThreadMessage {
    Start,
    Stop,
    Terminate,
}

pub struct AppState {
    running: bool,
    settings_panel: SettingsPanel,
    game: Arc<Mutex<Game>>,
    #[cfg(not(target_arch = "wasm32"))]
    game_thread: Option<JoinHandle<()>>,
    #[cfg(not(target_arch = "wasm32"))]
    tx: Sender<ThreadMessage>,
    claimed_amount: HashMap<u8, u32>,
    #[cfg(target_arch = "wasm32")]
    last_step_time: Instant,
}

pub struct App {
    pub state: AppState,
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let game = Arc::<Mutex<Game>>::default();
        let settings_panel = SettingsPanel::default();

        #[cfg(not(target_arch = "wasm32"))]
        let (tx, rx) = mpsc::channel::<ThreadMessage>();
        #[cfg(not(target_arch = "wasm32"))]
        let mut running = false;
        #[cfg(not(target_arch = "wasm32"))]
        let game_mutex = Arc::clone(&game);
        #[cfg(not(target_arch = "wasm32"))]
        let step_delay = Arc::clone(&settings_panel.step_delay);
        #[cfg(not(target_arch = "wasm32"))]
        let ctx = _cc.egui_ctx.clone();

        #[cfg(not(target_arch = "wasm32"))]
        let handle = thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        ThreadMessage::Start => {
                            running = true;
                        }
                        ThreadMessage::Stop => {
                            running = false;
                        }
                        ThreadMessage::Terminate => break,
                    },
                    Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {
                        if running {
                            thread::sleep(time::Duration::from_millis(
                                step_delay.load(Ordering::Relaxed),
                            ));

                            game_mutex
                                .lock()
                                .expect("Error while acquiring lock on game mutex")
                                .step();

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
                game,
                #[cfg(not(target_arch = "wasm32"))]
                game_thread: Some(handle),
                #[cfg(not(target_arch = "wasm32"))]
                tx,
                claimed_amount: HashMap::new(),
                #[cfg(target_arch = "wasm32")]
                last_step_time: Instant::now(),
            },
        }
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
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
                .send(ThreadMessage::Terminate)
                .expect("Error while sending Terminate ThreadMessage");
            self.state
                .game_thread
                .take()
                .expect("Error while take()-ing game thread")
                .join()
                .expect("Error while joining game thread");
        }

        #[cfg(target_arch = "wasm32")]
        if self.state.running {
            if self.state.last_step_time.elapsed()
                > Duration::from_millis(self.state.settings_panel.step_delay)
            {
                self.state
                    .game
                    .lock()
                    .expect("Error while acquiring lock on game mutex")
                    .step();

                self.state.last_step_time = Instant::now();
            }

            ui.ctx().request_repaint();
        }

        Panel::top("top_panel").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.visuals_mut().button_frame = false;

                self.bar_contents(ui, frame);
            });
        });

        let mut cmd = Command::Nothing;

        CentralPanel::default().show_inside(ui, |ui| {
            self.settings_panel(ui, frame, &mut cmd);

            self.game_panel(ui, frame);

            self.stats_window(ui, frame);
        });

        match cmd {
            Command::Nothing => {}
            Command::ApplyGridSize => {
                self.state
                    .game
                    .lock()
                    .expect("Error while acquiring lock on game mutex")
                    .resize(
                        self.state.settings_panel.width,
                        self.state.settings_panel.height,
                    );
            }
            Command::AddPlayer => {
                self.state
                    .game
                    .lock()
                    .expect("Error while acquiring lock on game mutex")
                    .add_player();
            }
            Command::RemovePlayer(id) => {
                self.state
                    .settings_panel
                    .players_settings
                    .retain(|player| player.id != id);

                self.state
                    .game
                    .lock()
                    .expect("Error while acquiring lock on game mutex")
                    .remove_player(id);
            }
            Command::SetStrategy(id, strategy) => {
                self.state
                    .game
                    .lock()
                    .expect("Error while acquiring lock on game mutex")
                    .set_player_strategy(
                        id,
                        STRATEGIES.get(&strategy).expect("Strategy not found"),
                    );
            }
            Command::MovePlayer(id, x, y) => {
                self.state
                    .game
                    .lock()
                    .expect("Error while acquiring lock on game mutex")
                    .move_player_to(x, y, id);
            }
            Command::DisablePlayer(id) => {
                self.state
                    .game
                    .lock()
                    .expect("Error while acquiring lock on game mutex")
                    .disable_player(id);
            }
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
                self.state.settings_panel.ui(
                    ui,
                    frame,
                    cmd,
                    &self
                        .state
                        .game
                        .lock()
                        .expect("Error while acquiring lock on game mutex")
                        .players,
                );
            });
    }

    fn bar_contents(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.add_space(4.0);

        widgets::global_theme_preference_switch(ui);

        ui.separator();

        ui.toggle_value(&mut self.state.settings_panel.open, "Settings");

        ui.separator();

        if ui.button("Reset").clicked() {
            self.state
                .game
                .lock()
                .expect("Error while acquiring lock on game mutex")
                .reset(&self.state.settings_panel.players_settings);
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
                    .send(ThreadMessage::Start)
                    .expect("Error while sending Start ThreadMessage");
            } else {
                self.state
                    .tx
                    .send(ThreadMessage::Stop)
                    .expect("Error while sending Stop ThreadMessage");
            }
        }
    }

    fn game_panel(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());

        self.state.claimed_amount.clear();

        let game = self
            .state
            .game
            .lock()
            .expect("Error while acquiring lock on game mutex");

        let width = game.width;
        let height = game.height;
        let mut offset_x = 0.;
        let mut offset_y = 0.;
        let cell_size;

        if response.rect.aspect_ratio() * height as f32 > width as f32 {
            cell_size = response.rect.height() / height as f32;

            offset_x = 0.5 * (response.rect.width() - cell_size * width as f32);
        } else {
            cell_size = response.rect.width() / width as f32;

            offset_y = 0.5 * (response.rect.height() - cell_size * height as f32);
        }

        for y in 0..height {
            for x in 0..width {
                let center = Pos2::new(
                    response.rect.left() + offset_x + (x as f32 + 0.5) * cell_size,
                    response.rect.top() + offset_y + (y as f32 + 0.5) * cell_size,
                );

                let cell = game.get_cell_at(x, y);

                let mut fill_color = match cell {
                    Cell::Empty => Color32::GRAY.gamma_multiply(0.2),
                    Cell::Player(id) | Cell::PlayerClaimed(id) => {
                        let rgb = self
                            .state
                            .settings_panel
                            .players_settings
                            .iter()
                            .find(|settings| settings.id == *id)
                            .expect("Player not found")
                            .color;

                        Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                    }
                };

                if let Cell::PlayerClaimed(_) = cell {
                    fill_color = fill_color.gamma_multiply(0.5);
                }

                match cell {
                    Cell::Empty => {}
                    Cell::Player(id) | Cell::PlayerClaimed(id) => {
                        let value = self.state.claimed_amount.get(id).unwrap_or(&0) + 1;
                        self.state.claimed_amount.insert(*id, value);
                    }
                }

                painter.rect(
                    Rect::from_center_size(center, Vec2::new(cell_size, cell_size)),
                    0.0,
                    fill_color,
                    Stroke::new(self.state.settings_panel.border_thickness, Color32::GRAY),
                    StrokeKind::Middle,
                );
            }
        }
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

                        for settings in &self.state.settings_panel.players_settings {
                            if settings.enabled {
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

                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            let taken = self
                                                .state
                                                .claimed_amount
                                                .get(&settings.id)
                                                .unwrap_or(&0);
                                            ui.label(format!(
                                                "{:.2}%",
                                                *taken as f32 / total as f32 * 100.
                                            ));
                                        });
                                    });
                                });
                            }
                        }
                    });
            });
    }
}
