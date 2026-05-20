#[cfg(not(target_arch = "wasm32"))]
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use eframe::Frame;
use egui::{Button, ComboBox, DragValue, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    game::{Player, Pos},
    strategies::STRATEGIES,
};

pub struct PlayerSettings {
    pub id: u8,
    pub color: [u8; 3],
    pub name: String,
    pub strategy: u8,
    pub x: usize,
    pub y: usize,
    pub enabled: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            id: 0,
            color: [255, 255, 255],
            name: "New player".to_owned(),
            strategy: 0,
            x: 0,
            y: 0,
            enabled: false,
        }
    }
}

pub struct SettingsPanel {
    pub open: bool,
    pub width: usize,
    pub height: usize,
    pub border_thickness: f32,
    pub players_settings: Vec<PlayerSettings>,
    #[cfg(not(target_arch = "wasm32"))]
    pub step_delay: Arc<AtomicU64>,
    #[cfg(target_arch = "wasm32")]
    pub step_delay: u64,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            open: true,
            width: 50,
            height: 50,
            border_thickness: 1.0,
            players_settings: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            step_delay: Arc::new(AtomicU64::new(50)),
            #[cfg(target_arch = "wasm32")]
            step_delay: 50,
        }
    }
}

#[derive(Default)]
pub enum Command {
    #[default]
    Nothing,
    ApplyGridSize,
    AddPlayer,
    RemovePlayer(u8),
    SetStrategy(u8, u8),
    MovePlayer(u8, usize, usize),
    DisablePlayer(u8),
}

#[expect(clippy::too_many_lines)]
impl SettingsPanel {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        _frame: &mut Frame,
        cmd: &mut Command,
        players: &Vec<Player>,
    ) {
        TableBuilder::new(ui)
            .id_salt("grid_settings")
            .column(Column::remainder())
            .column(Column::auto())
            .body(|mut body| {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Time between steps:");
                    });

                    row.col(|ui| {
                        ui.add(
                            #[cfg(not(target_arch = "wasm32"))]
                            DragValue::from_get_set(|val| {
                                if let Some(delay) = val {
                                    self.step_delay.store(delay as u64, Ordering::Relaxed);
                                }

                                self.step_delay.load(Ordering::Relaxed) as f64
                            })
                            .range(0..=1000)
                            .suffix("ms"),
                            #[cfg(target_arch = "wasm32")]
                            DragValue::new(&mut self.step_delay)
                                .range(0..=1000)
                                .suffix("ms"),
                        );
                    });
                });

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Width:");
                    });

                    row.col(|ui| {
                        ui.add(DragValue::new(&mut self.width));
                    });
                });

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Height:");
                    });

                    row.col(|ui| {
                        ui.add(DragValue::new(&mut self.height));
                    });
                });

                body.row(20.0, |mut row| {
                    row.col(|_| {});

                    row.col(|ui| {
                        if ui.add(Button::new("Apply")).clicked() {
                            *cmd = Command::ApplyGridSize;
                        }
                    });
                });

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Cell border thickness:");
                    });

                    row.col(|ui| {
                        ui.add(
                            DragValue::new(&mut self.border_thickness)
                                .speed(0.05)
                                .range(0..=2),
                        );
                    });
                });

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Players:");
                    });

                    row.col(|ui| {
                        if ui
                            .add_sized(ui.available_size(), Button::new("Add"))
                            .clicked()
                        {
                            *cmd = Command::AddPlayer;
                        }
                    });
                });
            });

        TableBuilder::new(ui)
            .id_salt("players_settings")
            .striped(true)
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .column(Column::auto())
            .body(|mut body| {
                for player in players {
                    let player_settings: Option<&mut PlayerSettings> =
                        self.players_settings.iter_mut().find(|p| p.id == player.id);

                    match player_settings {
                        Some(settings) => {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui.checkbox(&mut settings.enabled, "").changed() {
                                        if settings.enabled {
                                            *cmd = Command::MovePlayer(
                                                settings.id,
                                                settings.x,
                                                settings.y,
                                            );
                                        } else {
                                            *cmd = Command::DisablePlayer(settings.id);
                                        }
                                    }
                                });

                                row.col(|ui| {
                                    ui.add(TextEdit::singleline(&mut settings.name));
                                });

                                row.col(|ui| {
                                    let before = settings.strategy;

                                    ComboBox::from_id_salt(format!(
                                        "player_{}_strategy_selector",
                                        settings.id
                                    ))
                                    .selected_text(
                                        STRATEGIES
                                            .get(&settings.strategy)
                                            .expect("Strategy not found")
                                            .get_name(),
                                    )
                                    .show_ui(ui, |ui| {
                                        let mut strategy_ids =
                                            STRATEGIES.keys().clone().collect::<Vec<&u8>>();

                                        strategy_ids.sort();

                                        for strategy_id in strategy_ids {
                                            ui.selectable_value(
                                                &mut settings.strategy,
                                                *strategy_id,
                                                STRATEGIES[strategy_id].get_name(),
                                            );
                                        }
                                    });

                                    if before != settings.strategy {
                                        *cmd = Command::SetStrategy(settings.id, settings.strategy);
                                    }
                                });

                                row.col(|_| {});

                                row.col(|ui| {
                                    if ui.button("×").clicked() {
                                        *cmd = Command::RemovePlayer(settings.id);
                                    }
                                });
                            });

                            body.row(20.0, |mut row| {
                                row.col(|_| {});

                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("(");
                                        let x = ui.add(
                                            DragValue::new(&mut settings.x)
                                                .range(0..=self.width - 1),
                                        );
                                        ui.label(",");
                                        let y = ui.add(
                                            DragValue::new(&mut settings.y)
                                                .range(0..=self.height - 1),
                                        );
                                        ui.label(")");

                                        if (x.changed() || y.changed()) && settings.enabled {
                                            *cmd = Command::MovePlayer(
                                                settings.id,
                                                settings.x,
                                                settings.y,
                                            );
                                        }
                                    });
                                });

                                row.col(|ui| {
                                    ui.color_edit_button_srgb(&mut settings.color);
                                });
                            });
                        }
                        None => {
                            self.players_settings.push(PlayerSettings {
                                id: player.id,
                                x: player.position.unwrap_or(Pos::ZERO).x,
                                y: player.position.unwrap_or(Pos::ZERO).y,
                                enabled: player.position.is_some(),
                                ..Default::default()
                            });
                        }
                    }
                }
            });
    }
}
