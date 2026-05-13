use eframe::Frame;
use egui::{Button, ComboBox, DragValue, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    game::{Player, Pos},
    strategies::Strategies,
};

#[derive(Debug)]
pub struct PlayerSettings {
    pub id: u8,
    pub color: [u8; 3],
    pub name: String,
    pub strategy: Strategies,
    pub x: u16,
    pub y: u16,
    pub enabled: bool,
}

#[derive(Default)]
pub struct SettingsPanel {
    pub open: bool,
    pub width: u16,
    pub height: u16,
    pub players_settings: Vec<PlayerSettings>,
}

#[derive(Default)]
pub enum Command {
    #[default]
    Nothing,
    ApplyGridSize,
    AddPlayer,
    RemovePlayer(u8),
    SetStrategy(u8, Strategies),
    MovePlayer(u8, u16, u16),
    DisablePlayer(u8),
}

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
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .column(Column::auto())
            .body(|mut body| {
                for player in players {
                    let player_settings: Option<&mut PlayerSettings> = self
                        .players_settings
                        .iter_mut()
                        .filter(|p| p.id == player.id)
                        .next();

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
                                    .selected_text(settings.strategy.get().get_name())
                                    .show_ui(ui, |ui| {
                                        for strategy in Strategies::list_strategies() {
                                            ui.selectable_value(
                                                &mut settings.strategy,
                                                strategy,
                                                strategy.get().get_name(),
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
                                color: [255, 255, 255],
                                name: "New player".to_string(),
                                strategy: player.strategy,
                                x: player.position.unwrap_or(Pos::ZERO).x,
                                y: player.position.unwrap_or(Pos::ZERO).y,
                                enabled: player.position.is_some(),
                            });
                        }
                    }
                }
            });
    }
}
