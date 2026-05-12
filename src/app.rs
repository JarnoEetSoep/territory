use core::time;
use std::{collections::HashMap, sync::{Arc, Mutex, mpsc::{self, Sender, TryRecvError}}, thread::{self, JoinHandle}};

use egui::RichText;
use egui_extras::{Column, TableBuilder};

use crate::settings_panel::SettingsPanel;

pub enum ThreadMessage {
    Start,
    Stop,
    Terminate
}

pub struct AppState {
    running: bool,
    settings_panel: crate::settings_panel::SettingsPanel,
    game: Arc<Mutex<crate::game::Game>>,
    game_thread: Option<JoinHandle<()>>,
    tx: Sender<ThreadMessage>,
    bins: HashMap<u8, u32>
}

pub struct App {
    pub state: AppState
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let game = Arc::<Mutex<crate::game::Game>>::default();

        let (tx, rx) = mpsc::channel::<ThreadMessage>();
        let mut running = false;
        let game_mutex = Arc::clone(&game);

        let handle = thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(msg) => {
                    match msg {
                        ThreadMessage::Start => { running = true; },
                        ThreadMessage::Stop => { running = false; },
                        ThreadMessage::Terminate => break,
                    }
                },
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    if running {
                        thread::sleep(time::Duration::from_millis(50));

                        game_mutex.lock().unwrap().step();
                    }
                }
            }
        });

        Self {
            state: AppState {
                running: false,
                settings_panel: SettingsPanel::default(),
                game,
                game_thread: Some(handle),
                tx,
                bins: HashMap::new()
            },
        }
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            let fullscreen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ui.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        }

        if ui.input(|i| i.viewport().close_requested()) {
            self.state.tx.send(ThreadMessage::Terminate).unwrap();
            self.state.game_thread.take().unwrap().join().unwrap();
        }

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.visuals_mut().button_frame = false;

                self.bar_contents(ui, frame);
            });
        });

        let mut cmd = crate::settings_panel::Command::Nothing;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.settings_panel(ui, frame, &mut cmd);

            self.game_panel(ui, frame);

            self.stats_window(ui, frame);
        });

        match cmd {
            crate::settings_panel::Command::Nothing => {},
            crate::settings_panel::Command::ApplyGridSize => {
                self.state.game.lock().unwrap().resize(self.state.settings_panel.width, self.state.settings_panel.height);
            },
            crate::settings_panel::Command::AddPlayer => {
                self.state.game.lock().unwrap().add_player();
            },
            crate::settings_panel::Command::RemovePlayer(id) => {
                self.state.game.lock().unwrap().remove_player(id);
            },
            crate::settings_panel::Command::SetStrategy(id, strategy) => {
                self.state.game.lock().unwrap().set_player_strategy(id, strategy);
            },
            crate::settings_panel::Command::MovePlayer(id, x, y) => {
                self.state.game.lock().unwrap().move_player_to(x, y, id);
            },
            crate::settings_panel::Command::DisablePlayer(id) => {
                self.state.game.lock().unwrap().disable_player(id);
            }
        };

        ui.ctx().request_repaint_after_secs(1.0 / 30.0);
    }
}

impl App {
    fn settings_panel(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cmd: &mut crate::settings_panel::Command) {
        let is_open = self.state.settings_panel.open || ui.memory(|mem| mem.everything_is_visible());

        egui::Panel::left("settings_panel")
            .resizable(true)
            .show_animated_inside(ui, is_open, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Settings");
                });

                ui.separator();
                self.state.settings_panel.ui(ui, frame, cmd, &self.state.game.lock().unwrap().players);
            });
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.add_space(4.0);

        egui::widgets::global_theme_preference_switch(ui);

        ui.separator();
        
        ui.toggle_value(&mut self.state.settings_panel.open, "Settings");

        ui.separator();

        let content = match (self.state.running, ui.visuals().dark_mode) {
            (true, true) => RichText::new("⏸").color(egui::Color32::LIGHT_RED),
            (false, true) => RichText::new("▶").color(egui::Color32::LIGHT_GREEN),
            (true, false) => RichText::new("⏸").color(egui::Color32::RED),
            (false, false) => RichText::new("▶").color(egui::Color32::GREEN)
        };
        
        if ui.button(content).clicked() {
            self.state.running = !self.state.running;

            if self.state.running {
                self.state.tx.send(ThreadMessage::Start).unwrap();
            } else {
                self.state.tx.send(ThreadMessage::Stop).unwrap();
            }
        }
    }

    fn game_panel(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click()
        );

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
                let center = egui::Pos2::new(
                    response.rect.left() + offset_x + (f32::from(x) + 0.5) * cell_size,
                    response.rect.top() + offset_y + (f32::from(y) + 0.5) * cell_size
                );

                let cell = game.get_cell_at(x, y);

                let fill_color = match cell {
                    crate::game::Cell::Empty => egui::Color32::WHITE,
                    crate::game::Cell::Player(id) => {
                        let rgb = self.state.settings_panel.players_settings.iter().filter(|settings| settings.id == *id).next().unwrap().color;

                        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                    },
                    crate::game::Cell::PlayerClaimed(id) => {
                        let rgb = self.state.settings_panel.players_settings.iter().filter(|settings| settings.id == *id).next().unwrap().color;

                        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]).gamma_multiply(0.5)
                    },
                };

                match cell {
                    crate::game::Cell::Empty => {},
                    crate::game::Cell::Player(id) | crate::game::Cell::PlayerClaimed(id) => {
                        let value = self.state.bins.get(id).unwrap_or(&0) + 1;
                        self.state.bins.insert(*id, value);
                    }
                }
                
                painter.rect(egui::Rect::from_center_size(
                    center,
                    egui::Vec2::new(cell_size, cell_size)),
                    0.0,
                    fill_color,
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                    egui::StrokeKind::Middle
                );
            }
        }
    }

    fn stats_window(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Window::new("Stats").resizable(false).show(ui.ctx(), |ui| {
            TableBuilder::new(ui)
                .id_salt("stats")
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .body(|mut body| {
                    let total = u32::from(self.state.settings_panel.width) * u32::from(self.state.settings_panel.height);

                    for settings in &self.state.settings_panel.players_settings {
                        if settings.enabled {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    let (r, g, b) = (settings.color[0], settings.color[1], settings.color[2]);
                                    ui.colored_label(egui::Color32::from_rgb(r, g, b), "⬛");
                                });

                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}:", settings.name));
                                    });
                                });

                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        let taken = self.state.bins.get(&settings.id).unwrap_or(&0);
                                        ui.label(format!("{:.2}%", f64::from(*taken) / f64::from(total) * 100.));
                                    });
                                });
                            });
                        }
                    }
                });
        });
    }
}
