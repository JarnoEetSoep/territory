#[cfg(not(target_arch = "wasm32"))]
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use eframe::Frame;
use egui::{Button, ComboBox, DragValue, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

use crate::strategies::STRATEGIES;

#[derive(Clone, Copy, Default)]
pub struct CorePlayerSettings {
    pub id: u8,
    pub x: usize,
    pub y: usize,
    pub enabled: bool,
}

pub struct PlayerSettings {
    pub core_settings: CorePlayerSettings,
    pub color: [u8; 3],
    pub name: String,
    pub strategy: usize,
    pub current_position: Option<(usize, usize)>,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            core_settings: CorePlayerSettings::default(),
            color: [255, 255, 255],
            name: "New player".to_owned(),
            strategy: 0,
            current_position: None,
        }
    }
}

pub struct SettingsPanel {
    pub open: bool,
    pub width: usize,
    pub height: usize,
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
    ApplyGridSize(usize, usize),
    AddPlayer,
    RemovePlayer(u8),
    SetStrategy(u8, usize),
    MovePlayer(u8, usize, usize),
    DisablePlayer(u8),
    Reset(Vec<CorePlayerSettings>),
    ColorChanged(u8),
}

#[expect(clippy::too_many_lines)]
impl SettingsPanel {
    pub fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame, cmd: &mut Command) {
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
                            *cmd = Command::ApplyGridSize(self.width, self.height);

                            self.players_settings
                                .iter_mut()
                                .for_each(|player| player.core_settings.enabled = false);
                        }
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
                for settings in &mut self.players_settings {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            if ui
                                .checkbox(&mut settings.core_settings.enabled, "")
                                .changed()
                            {
                                if settings.core_settings.enabled {
                                    *cmd = Command::MovePlayer(
                                        settings.core_settings.id,
                                        settings.core_settings.x,
                                        settings.core_settings.y,
                                    );
                                } else {
                                    *cmd = Command::DisablePlayer(settings.core_settings.id);
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
                                settings.core_settings.id
                            ))
                            .selected_text(
                                STRATEGIES
                                    .get(settings.strategy)
                                    .expect("Strategy not found")
                                    .get_name(),
                            )
                            .show_ui(ui, |ui| {
                                for (id, strategy) in STRATEGIES.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut settings.strategy,
                                        id,
                                        strategy.get_name(),
                                    );
                                }
                            });

                            if before != settings.strategy {
                                *cmd = Command::SetStrategy(
                                    settings.core_settings.id,
                                    settings.strategy,
                                );
                            }
                        });

                        row.col(|_| {});

                        row.col(|ui| {
                            if ui.button("×").clicked() {
                                *cmd = Command::RemovePlayer(settings.core_settings.id);
                            }
                        });
                    });

                    body.row(20.0, |mut row| {
                        row.col(|_| {});

                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("(");
                                let x = ui.add(
                                    DragValue::new(&mut settings.core_settings.x)
                                        .range(0..=self.width - 1),
                                );
                                ui.label(",");
                                let y = ui.add(
                                    DragValue::new(&mut settings.core_settings.y)
                                        .range(0..=self.height - 1),
                                );
                                ui.label(")");

                                if (x.changed() || y.changed()) && settings.core_settings.enabled {
                                    *cmd = Command::MovePlayer(
                                        settings.core_settings.id,
                                        settings.core_settings.x,
                                        settings.core_settings.y,
                                    );
                                }
                            });
                        });

                        let color_before = settings.color;

                        row.col(|ui| {
                            ui.color_edit_button_srgb(&mut settings.color);
                        });

                        if settings.color != color_before {
                            *cmd = Command::ColorChanged(settings.core_settings.id);
                        }
                    });
                }
            });
    }
}
