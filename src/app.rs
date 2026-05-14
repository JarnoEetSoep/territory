use core::time;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
};

use egui::{
    CentralPanel, Color32, MenuBar, Panel, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
    Vec2, Window, widgets,
};
#[cfg(not(target_arch = "wasm32"))]
use egui::{Key, Modifiers};
use egui_extras::{Column, TableBuilder};

use crate::{
    game::{Cell, Game},
    settings_panel::{Command, SettingsPanel},
};

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
    bins: HashMap<u8, u32>,
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

        #[cfg(not(target_arch = "wasm32"))]
        let (tx, rx) = mpsc::channel::<ThreadMessage>();
        #[cfg(not(target_arch = "wasm32"))]
        let mut running = false;
        #[cfg(not(target_arch = "wasm32"))]
        let game_mutex = Arc::clone(&game);

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
                            thread::sleep(time::Duration::from_millis(50));

                            game_mutex.lock().unwrap().step();
                        }
                    }
                }
            }
        });

        Self {
            state: AppState {
                running: false,
                settings_panel: SettingsPanel::default(),
                game,
                #[cfg(not(target_arch = "wasm32"))]
                game_thread: Some(handle),
                #[cfg(not(target_arch = "wasm32"))]
                tx,
                bins: HashMap::new(),
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
            self.state.tx.send(ThreadMessage::Terminate).unwrap();
            self.state.game_thread.take().unwrap().join().unwrap();
        }

        #[cfg(target_arch = "wasm32")]
        if self.state.running {
            self.state.game.lock().unwrap().step();
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
                self.state.game.lock().unwrap().resize(
                    self.state.settings_panel.width,
                    self.state.settings_panel.height,
                );
            }
            Command::AddPlayer => {
                self.state.game.lock().unwrap().add_player();
            }
            Command::RemovePlayer(id) => {
                self.state.game.lock().unwrap().remove_player(id);
            }
            Command::SetStrategy(id, strategy) => {
                self.state
                    .game
                    .lock()
                    .unwrap()
                    .set_player_strategy(id, strategy);
            }
            Command::MovePlayer(id, x, y) => {
                self.state.game.lock().unwrap().move_player_to(x, y, id);
            }
            Command::DisablePlayer(id) => {
                self.state.game.lock().unwrap().disable_player(id);
            }
        };

        ui.ctx().request_repaint_after_secs(0.05);
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
                    &self.state.game.lock().unwrap().players,
                );
            });
    }

    fn bar_contents(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.add_space(4.0);

        widgets::global_theme_preference_switch(ui);

        ui.separator();

        ui.toggle_value(&mut self.state.settings_panel.open, "Settings");

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
                self.state.tx.send(ThreadMessage::Start).unwrap();
            } else {
                self.state.tx.send(ThreadMessage::Stop).unwrap();
            }
        }
    }

    fn game_panel(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());

        self.state.bins.clear();

        let game = self.state.game.lock().unwrap();

        let width = game.width;
        let height = game.height;
        let mut offset_x = 0.;
        let mut offset_y = 0.;
        let cell_size;

        if response.rect.aspect_ratio() * f32::from(height) > f32::from(width) {
            cell_size = response.rect.height() / f32::from(height);

            offset_x = 0.5 * (response.rect.width() - cell_size * f32::from(width));
        } else {
            cell_size = response.rect.width() / f32::from(width);

            offset_y = 0.5 * (response.rect.height() - cell_size * f32::from(height));
        }

        for y in 0..height {
            for x in 0..width {
                let center = Pos2::new(
                    response.rect.left() + offset_x + (f32::from(x) + 0.5) * cell_size,
                    response.rect.top() + offset_y + (f32::from(y) + 0.5) * cell_size,
                );

                let cell = game.get_cell_at(x, y);

                let fill_color = match cell {
                    Cell::Empty => Color32::WHITE,
                    Cell::Player(id) => {
                        let rgb = self
                            .state
                            .settings_panel
                            .players_settings
                            .iter()
                            .filter(|settings| settings.id == *id)
                            .next()
                            .unwrap()
                            .color;

                        Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                    }
                    Cell::PlayerClaimed(id) => {
                        let rgb = self
                            .state
                            .settings_panel
                            .players_settings
                            .iter()
                            .filter(|settings| settings.id == *id)
                            .next()
                            .unwrap()
                            .color;

                        Color32::from_rgb(rgb[0], rgb[1], rgb[2]).gamma_multiply(0.5)
                    }
                };

                match cell {
                    Cell::Empty => {}
                    Cell::Player(id) | Cell::PlayerClaimed(id) => {
                        let value = self.state.bins.get(id).unwrap_or(&0) + 1;
                        self.state.bins.insert(*id, value);
                    }
                }

                painter.rect(
                    Rect::from_center_size(center, Vec2::new(cell_size, cell_size)),
                    0.0,
                    fill_color,
                    Stroke::new(1.0, Color32::GRAY),
                    StrokeKind::Middle,
                );
            }
        }
    }

    fn stats_window(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        Window::new("Stats").resizable(false).show(ui.ctx(), |ui| {
            TableBuilder::new(ui)
                .id_salt("stats")
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .body(|mut body| {
                    let total = u32::from(self.state.settings_panel.width)
                        * u32::from(self.state.settings_panel.height);

                    for settings in &self.state.settings_panel.players_settings {
                        if settings.enabled {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    let (r, g, b) =
                                        (settings.color[0], settings.color[1], settings.color[2]);
                                    ui.colored_label(Color32::from_rgb(r, g, b), "⬛");
                                });

                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}:", settings.name));
                                    });
                                });

                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        let taken = self.state.bins.get(&settings.id).unwrap_or(&0);
                                        ui.label(format!(
                                            "{:.2}%",
                                            f64::from(*taken) / f64::from(total) * 100.
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
