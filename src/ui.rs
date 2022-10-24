use crate::enemy::enemy_difficulty::*;
use crate::*;
use bevy::app::AppExit;
use std::process::exit;

use crate::egui::style::Margin;
use crate::egui::FontFamily::Name;
use crate::input::input_manager::UpgradeMenuEvent;
use bevy::prelude::*;
use bevy_egui::egui::*;
use bevy_egui::*;
use bevy_rapier2d::na::DimAdd;
use egui::{FontFamily, FontId, RichText, TextStyle};
use iyes_loopless::prelude::*;
use crate::sound::SoundEffects;

pub struct UiPlugin;

//const SELECTED_COLOR: Color32 = Color32::from_rgb(94 / 3, 255 / 3, 169 / 3);
//const DESELECTED_COLOR: Color32 = Color32::from_rgb(94 / 10, 255 / 10, 169 / 10);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::GameSetupOnce, setup_ui);
        //background ui systems
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::MainMenu)
                .before("main_ui")
                .with_system(outside_backgrounds)
                .into(),
        )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Pause)
                    .before("main_ui")
                    .with_system(outside_backgrounds)
                    .with_system(pause_ui)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .before("main_ui")
                    .with_system(outside_backgrounds)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Lose)
                    .before("main_ui")
                    .with_system(outside_backgrounds)
                    .into(),
            );
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("main_ui")
                .with_system(playing_ui)
                .into(),
        )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Pause)
                    .label("main_ui")
                    .with_system(playing_ui)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Lose)
                    .label("main_ui")
                    .with_system(lose_ui)
                    .into(),
            );

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::MainMenu)
                .label("main_ui")
                .with_system(main_menu_ui)
                .into(),
        );
    }
}

#[inline]
fn small_button_font() -> TextStyle {
    TextStyle::Name("SmallButtonText".into())
}

#[inline]
fn heading3() -> TextStyle {
    TextStyle::Name("ContextHeading".into())
}

fn setup_ui(mut egui_context: ResMut<EguiContext>) {
    let mut fonts = FontDefinitions::default();
    let mut style = (*egui_context.ctx_mut().style()).clone();
    fonts.font_data.insert(
        "main_font".to_owned(),
        FontData::from_static(include_bytes!("../assets/OpenSans-ExtraBold.ttf")),
    ); // .ttf and .otf supported
    // Large button text:
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "main_font".to_owned());

    let (font_family, _) = fonts
        .families
        .get_key_value(&FontFamily::Proportional)
        .unwrap();
    let font_id = FontId {
        size: 24.0,
        family: font_family.clone(),
    };

    let small_button_font_id = FontId {
        size: 20.0,
        family: font_family.clone(),
    };

    egui_context.ctx_mut().set_fonts(fonts);
    style.text_styles.insert(TextStyle::Body, font_id.clone());
    style.text_styles.insert(TextStyle::Button, font_id.clone());
    style
        .text_styles
        .insert(small_button_font(), small_button_font_id.clone());

    egui_context.ctx_mut().set_style(style);
}

fn outside_backgrounds(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    sprites: Res<AssetHolder>,
) {
    let wnd = windows.get_primary().unwrap();

    let my_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 255),
        stroke: Stroke::new(0., Color32::WHITE),
        ..default()
    };

    let logo = egui_context.add_image(sprites.logo.clone_weak());

    SidePanel::left("left_background")
        .frame(my_frame)
        .resizable(false)
        .min_width((wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.))
        .max_width((wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.))
        .show(egui_context.ctx_mut(), |ui| {});

    SidePanel::right("right_background")
        .frame(my_frame)
        .resizable(false)
        .min_width((wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.))
        .max_width((wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.))
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.image(logo, [384.0, 192.0]);
            });
        });
}

fn main_menu_ui(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    mut player_stats: ResMut<PlayerStats>,
    mut sound_effect_writer: EventWriter<SoundEffects>,
) {
    let wnd = windows.get_primary().unwrap();

    let my_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 255),
        stroke: Stroke::new(0., Color32::WHITE),
        ..default()
    };

    egui::Window::new("pause_screen")
        .frame(my_frame)
        .anchor(
            Align2::CENTER_BOTTOM,
            egui::Vec2 {
                x: 0.0,
                y: -(wnd.height() / 2.7),
            },
        )
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.group(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.label(&format!("SPACE COMMANDER"));
                        //ui.label(&format!("HIGH SCORE: {}", player_stats.locked_score));
                    });
                });
                ui.spacing_mut().item_spacing.y = 32.;
            });
            // options below the main panel with system stuff
            ui.columns(2, |ui| {
                let menu_button =
                    ui[0].add_sized([80., 26.], egui::Button::new(RichText::new("PLAY")));
                if menu_button.clicked() {
                    commands.insert_resource(NextState(GameState::Playing));
                    sound_effect_writer.send(SoundEffects::NormalButton);
                };
                let menu_button =
                    ui[1].add_sized([80., 26.], egui::Button::new(RichText::new("QUIT")));
                if menu_button.clicked() {
                    sound_effect_writer.send(SoundEffects::NormalButton);
                    quit_game(exit);
                };
            });
        });
}

fn playing_ui(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    sprites: Res<AssetHolder>,
    player_stats: Res<PlayerStats>,
    enemy_stats: Res<EnemyStats>,
    mut commands: Commands,
    mut sound_effect_writer: EventWriter<SoundEffects>,
) {
    let wnd = windows.get_primary().unwrap();

    let health = egui_context.add_image(sprites.health.clone_weak());
    let health_empty = egui_context.add_image(sprites.health_empty.clone_weak());

    let my_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 0),
        stroke: Stroke::new(0., Color32::WHITE),
        ..default()
    };

    //left side
    egui::Window::new("PLANET")
        .frame(my_frame)
        .anchor(Align2::LEFT_CENTER, vec2(-(wnd.height() as f32 / 2.57), 0.))
        //.fixed_pos(pos2(0.0, 16.0))
        .fixed_size(egui::Vec2 {
            x: (wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.),
            y: 500.,
        })
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            if ui.button(RichText::new("PAUSE").text_style(small_button_font())).clicked() {
                commands.insert_resource(NextState(GameState::Pause));
                sound_effect_writer.send(SoundEffects::NormalButton);
            }
            ui.vertical_centered(|ui| {
                let menu_button = ui.add_sized(
                    [80., 26.],
                    egui::Button::new(RichText::new("PAUSE").text_style(small_button_font())),
                );
                if menu_button.clicked() {
                    commands.insert_resource(NextState(GameState::Pause));
                };
            });
            ui.vertical_centered_justified(|ui| {
                ui.spacing_mut().item_spacing.y = 8.;
                ui.group(|ui| {
                    ui.label("ENERGY");
                    ui.add(
                        ProgressBar::new(
                            (player_stats.time_till_next_energy - 0.)
                                / (player_stats.energy_recharge_rate.0 - 0.),
                        )
                            .text(&format!("   +{} Energy", player_stats.energy_per_recharge)),
                    );
                    ui.label(&format!(
                        "ENERGY: {}/{}",
                        player_stats.current_energy, player_stats.max_energy
                    ));
                });
                ui.group(|ui| {
                    ui.label("HEALTH");
                    ui.horizontal_wrapped(|ui| {
                        for i in 0..player_stats.max_health {
                            if i < player_stats.current_health {
                                ui.image(health, [(16 * 3) as f32, (16 * 3) as f32]);
                            } else {
                                ui.image(health_empty, [(16 * 3) as f32, (16 * 3) as f32]);
                            }
                        }
                    });
                });
            });
        });

    //right side
    egui::Window::new("GAME STATS")
        .frame(my_frame)
        .anchor(Align2::RIGHT_CENTER, vec2((wnd.height() as f32 / 2.57), 0.))
        .fixed_size(egui::Vec2 {
            x: (wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.),
            y: 500.,
        })
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.spacing_mut().item_spacing.y = 8.;
                ui.label(&format!("SCORE: {}", player_stats.locked_score));
                ui.label(&format!("POINTS: {}", player_stats.score));
                ui.label(&format!(
                    "ENEMIES ALIVE: {}",
                    enemy_stats.current_enemy_amount
                ));
            });
        });
}

fn pause_ui(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    mut player_stats: ResMut<PlayerStats>,
    mut sound_effect_writer: EventWriter<SoundEffects>,
) {
    let wnd = windows.get_primary().unwrap();

    let my_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 255),
        stroke: Stroke::new(0., Color32::WHITE),
        ..default()
    };

    let game_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 255),
        stroke: Stroke::new(5., Color32::WHITE),
        inner_margin: Margin {
            left: 25.,
            right: 25.,
            top: 25.,
            bottom: 25.,
        },
        ..default()
    };

    egui::Window::new("pause_screen")
        .frame(my_frame)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.group(|ui| {
                    ui.label(&format!("PAUSED"));
                });
                ui.group(|ui| {
                    ui.columns(3, |ui| {
                        ui[0].set_max_height(40.);
                        ui[0].horizontal_centered(|ui| {
                            ui.label(&format!("POINTS: {}", player_stats.score));
                        });
                        ui[1].set_max_height(40.);
                        ui[1].horizontal_centered(|ui| {
                            ui.label(&format!("SCORE: {}", player_stats.locked_score));
                        });
                        ui[2].set_max_height(40.);
                        ui[2].horizontal_centered(|ui| {
                            //LOCK POINTS BUTTON
                            let button = ui.group(|ui| {
                                ui.set_min_width(100.);
                                ui.label(
                                    RichText::new("LOCK POINTS").text_style(small_button_font()),
                                );
                            });
                            let max_energy_button = button.response.interact(Sense::click());
                            let max_energy_button = max_energy_button.on_hover_text(
                                RichText::new("Convert all points into locked score")
                                    .text_style(small_button_font()),
                            );
                            if max_energy_button.clicked() {
                                player_stats.lock_remaining_score();
                            }
                        });
                    });
                });
                //ui.spacing_mut().item_spacing.y = 32.;

                ui.group(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.label(&format!("UPGRADE"));
                    });
                    //TODO get the background color for group working

                    ui.group(|ui| {
                        ui.columns(3, |ui| {
                            //ui.horizontal_wrapped(|ui| {
                            ui[0].vertical_centered(|ui| {
                                ui.set_max_height(50.);

                                //MAX ENERGY
                                let button = ui.group(|ui| {
                                    ui.set_min_width(100.);
                                    ui.label(
                                        RichText::new("Max Energy").text_style(small_button_font()),
                                    );
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "+1 Max Energy | Cost: {}",
                                        player_stats.max_energy_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_max_energy();
                                }
                            });

                            //ENERGY RECHARGE
                            ui[0].vertical_centered(|ui| {
                                ui.set_max_height(50.);
                                ui.set_min_width(100.);

                                let button = ui.group(|ui| {
                                    ui.label(
                                        RichText::new("Energy Recharge")
                                            .text_style(small_button_font()),
                                    );
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "+1 Energy per {} Seconds | Cost: {}",
                                        player_stats.energy_recharge_rate.0,
                                        player_stats.energy_recharge_rate_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_energy_charge();
                                }
                            });

                            //MAX HEALTH
                            ui[1].vertical_centered(|ui| {
                                ui.set_max_height(50.);
                                ui.set_min_width(100.);

                                let button = ui.group(|ui| {
                                    ui.label(
                                        RichText::new("Max Health").text_style(small_button_font()),
                                    );
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "+1 Max Health | Cost: {}",
                                        player_stats.max_health_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_max_health();
                                }
                            });

                            //HEAL
                            ui[1].vertical_centered(|ui| {
                                ui.set_max_height(20.);
                                ui.set_min_width(100.);

                                let button = ui.group(|ui| {
                                    ui.label(RichText::new("Heal").text_style(small_button_font()));
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "Heals 1 Health | Cost: {}",
                                        player_stats.current_health_increase_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.plus_current_health();
                                }
                            });

                            //SCANS / shield / missile
                            ui[2].vertical_centered(|ui| {
                                ui.set_max_height(20.);
                                ui.set_min_width(100.);

                                let button = ui.group(|ui| {
                                    ui.label(
                                        RichText::new("Faster Scans")
                                            .text_style(small_button_font()),
                                    );
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "Increases scan speed by {} | Cost: {}",
                                        player_stats.scan_speed.2,
                                        player_stats.scan_speed_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_scan_speed();
                                }
                            });
                            ui[2].vertical_centered(|ui| {
                                ui.set_max_height(20.);
                                ui.set_min_width(100.);

                                let button = ui.group(|ui| {
                                    ui.label(
                                        RichText::new("Shield Time")
                                            .text_style(small_button_font()),
                                    );
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "Increases shield time by {} | Cost: {}",
                                        1, player_stats.shield_time_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_shield_time();
                                }
                            });
                            ui[2].vertical_centered(|ui| {
                                ui.set_max_height(20.);
                                ui.set_min_width(100.);

                                let button = ui.group(|ui| {
                                    ui.label(
                                        RichText::new("Missile Speed")
                                            .text_style(small_button_font()),
                                    );
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "Increases missile speed by {} | Cost: {}",
                                        player_stats.missile_speed.2,
                                        player_stats.missile_speed_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_missile_speed();
                                }
                            });
                        });
                    });
                });

                ui.group(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.label(&format!("SUPER UPGRADES"));
                    });
                    ui.group(|ui| {
                        ui.columns(4, |ui| {
                            //ui.horizontal_wrapped(|ui| {
                            ui[0].vertical_centered(|ui| {
                                ui.set_max_height(50.);

                                //CLUSTER MISSILE
                                let button = ui.group(|ui| {
                                    ui.set_min_width(100.);
                                    if player_stats.is_cluster_missile_upgrade {
                                        ui.label(
                                            RichText::new("Cluster Missile").text_style(small_button_font()).strikethrough().color(Color32::from_rgba_unmultiplied(0, 200, 0, 255)),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new("Cluster Missile").text_style(small_button_font()).color(Color32::from_rgba_unmultiplied(200, 0, 0, 255)),
                                        );
                                    }
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "Fires 4 missiles in an aoe around the target point | Cost: {}",
                                        player_stats.cluster_missile_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_cluster_missile();
                                }
                            });
                            ui[1].vertical_centered(|ui| {
                                ui.set_max_height(50.);

                                //ENERGY VAMPIRE
                                let button = ui.group(|ui| {
                                    ui.set_min_width(100.);
                                    if player_stats.is_energy_vampire_upgrade {
                                        ui.label(
                                            RichText::new("Energy Vampire").text_style(small_button_font()).strikethrough().color(Color32::from_rgba_unmultiplied(0, 200, 0, 255)),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new("Energy Vampire").text_style(small_button_font()).color(Color32::from_rgba_unmultiplied(200, 0, 0, 255)),
                                        );
                                    }
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "If a missile kills at least one enemy, refund one energy | Cost: {}",
                                        player_stats.energy_vampire_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_energy_vampire();
                                }
                            });
                            ui[2].vertical_centered(|ui| {
                                ui.set_max_height(50.);

                                //DYING SCANNERS
                                let button = ui.group(|ui| {
                                    ui.set_min_width(100.);
                                    if player_stats.is_dying_scanners_upgrade {
                                        ui.label(
                                            RichText::new("Dying Scanners").text_style(small_button_font()).strikethrough().color(Color32::from_rgba_unmultiplied(0, 200, 0, 255)),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new("Dying Scanners").text_style(small_button_font()).color(Color32::from_rgba_unmultiplied(200, 0, 0, 255)),
                                        );
                                    }
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "Killing an enemy releases a small scan around their death point | Cost: {}",
                                        player_stats.dying_scanners_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_dying_scanners();
                                }
                            });
                            ui[3].vertical_centered(|ui| {
                                ui.set_max_height(50.);

                                //LARGER MISSILES
                                let button = ui.group(|ui| {
                                    ui.set_min_width(100.);
                                    if player_stats.is_larger_missiles_upgrade {
                                        ui.label(
                                            RichText::new("Larger Missiles").text_style(small_button_font()).strikethrough().color(Color32::from_rgba_unmultiplied(0, 200, 0, 255)),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new("Larger Missiles").text_style(small_button_font()).color(Color32::from_rgba_unmultiplied(200, 0, 0, 255)),
                                        );
                                    }
                                });
                                let max_energy_button = button.response.interact(Sense::click());
                                let max_energy_button = max_energy_button.on_hover_text(
                                    RichText::new(format!(
                                        "Larger explosion radius for all missiles | Cost: {}",
                                        player_stats.larger_missiles_upgrade_cost
                                    ))
                                        .text_style(small_button_font()),
                                );
                                if max_energy_button.clicked() {
                                    player_stats.upgrade_larger_missiles();
                                }
                            });
                        });
                    });
                });

                // options below the main panel with system stuff
                ui.columns(3, |ui| {
                    let menu_button = ui[0].add_sized(
                        [80., 26.],
                        egui::Button::new(RichText::new("RESUME").text_style(small_button_font())),
                    );
                    if menu_button.clicked() {
                        commands.insert_resource(NextState(GameState::Playing));
                    };

                    let menu_button = ui[1].add_sized(
                        [80., 26.],
                        egui::Button::new(
                            RichText::new("MAIN MENU").text_style(small_button_font()),
                        ),
                    );
                    if menu_button.clicked() {
                        commands.insert_resource(NextState(GameState::MainMenu));
                    };

                    let menu_button = ui[2].add_sized(
                        [80., 26.],
                        egui::Button::new(RichText::new("QUIT").text_style(small_button_font())),
                    );
                    if menu_button.clicked() {
                        quit_game(exit);
                    };
                });
            });
        });
}

fn lose_ui(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    mut player_stats: ResMut<PlayerStats>,
    mut sound_effect_writer: EventWriter<SoundEffects>,
) {
    let wnd = windows.get_primary().unwrap();

    let my_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 255),
        stroke: Stroke::new(0., Color32::WHITE),
        ..default()
    };

    egui::Window::new("pause_screen")
        .frame(my_frame)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.group(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.label(&format!("GAME OVER"));
                        ui.label(&format!("FINAL SCORE: {}", player_stats.locked_score));
                    });
                });
                ui.spacing_mut().item_spacing.y = 32.;
            });
            // options below the main panel with system stuff
            ui.columns(2, |ui| {
                let menu_button = ui[0].add_sized(
                    [80., 26.],
                    egui::Button::new(RichText::new("RESTART").text_style(small_button_font())),
                );
                if menu_button.clicked() {
                    commands.insert_resource(NextState(GameState::MainMenu));
                };
                let menu_button = ui[1].add_sized(
                    [80., 26.],
                    egui::Button::new(RichText::new("QUIT").text_style(small_button_font())),
                );
                if menu_button.clicked() {
                    quit_game(exit);
                };
            });
        });
}

fn quit_game(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}
