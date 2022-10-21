use crate::enemy::enemy_difficulty::*;
use crate::*;
use bevy::app::AppExit;
use std::process::exit;

use crate::egui::style::Margin;
use crate::input::input_manager::UpgradeMenuEvent;
use bevy::prelude::*;
use bevy_egui::egui::*;
use bevy_egui::*;
use bevy_rapier2d::na::DimAdd;
use egui::{FontFamily, FontId, RichText, TextStyle};
use iyes_loopless::prelude::*;

pub struct UiPlugin;

//const SELECTED_COLOR: Color32 = Color32::from_rgb(94 / 3, 255 / 3, 169 / 3);
//const DESELECTED_COLOR: Color32 = Color32::from_rgb(94 / 10, 255 / 10, 169 / 10);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::GameSetupOnce, setup_ui);
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
            );
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("main_ui")
                .with_system(main_ui)
                .into(),
        )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Pause)
                    .label("main_ui")
                    .with_system(main_ui)
                    .into(),
            );
    }
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

    egui_context.ctx_mut().set_fonts(fonts);
    style.text_styles.insert(TextStyle::Body, font_id.clone());
    style.text_styles.insert(TextStyle::Button, font_id.clone());

    egui_context.ctx_mut().set_style(style);
}

fn main_ui(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    sprites: Res<AssetHolder>,
    player_stats: Res<PlayerStats>,
    enemy_stats: Res<EnemyStats>,
) {
    let wnd = windows.get_primary().unwrap();

    let my_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 0),
        stroke: Stroke::new(0., Color32::WHITE),
        ..default()
    };

    egui::Window::new("PLANET")
        .frame(my_frame)
        .fixed_pos(pos2(0.0, 16.0))
        .fixed_size(egui::Vec2 {
            x: (wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.),
            y: 500.,
        })
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            let mut style = ui.style_mut();
            style.visuals.override_text_color = Some(Color32::from_rgb(255, 255, 255));
            //style.visuals.widgets.ac.bg_stroke = Color32::from_rgb(76, 153, 64);
            style.visuals.widgets.open.fg_stroke = Stroke::new(10., Color32::from_rgb(76, 153, 64));

            style.visuals.widgets.open.bg_fill = Color32::from_rgb(76, 153, 64);

            //style.visuals.widgets.active.bg_fill = Color32::from_rgb(76, 153, 64);
            //style.visuals.widgets.active.bg_fill = Color32::from_rgb(76, 153, 64);

            ui.vertical_centered_justified(|ui| {
                ui.spacing_mut().item_spacing.y = 8.;
                ui.add(ProgressBar::new(
                    (player_stats.time_till_next_energy - 0.)
                        / (player_stats.energy_recharge_rate as f32 - 0.),
                ));
                ui.label(&format!(
                    "ENERGY: {}/{}",
                    player_stats.current_energy, player_stats.max_energy
                ))
            });
        });

    egui::Window::new("GAME STATS")
        .frame(my_frame)
        .fixed_pos(pos2(
            ((wnd.width() as f32 / 2.) + (wnd.height() as f32 / 2.)),
            192. + 16.,
        ))
        .fixed_size(egui::Vec2 {
            x: (wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.),
            y: 500.,
        })
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            let mut style = ui.style_mut();
            style.visuals.override_text_color = Some(Color32::from_rgb(255, 255, 255));
            //style.visuals.widgets.ac.bg_stroke = Color32::from_rgb(76, 153, 64);
            style.visuals.widgets.open.fg_stroke = Stroke::new(10., Color32::from_rgb(76, 153, 64));

            style.visuals.widgets.open.bg_fill = Color32::from_rgb(76, 153, 64);

            //style.visuals.widgets.active.bg_fill = Color32::from_rgb(76, 153, 64);
            //style.visuals.widgets.active.bg_fill = Color32::from_rgb(76, 153, 64);
            ui.vertical_centered_justified(|ui| {
                ui.spacing_mut().item_spacing.y = 8.;
                ui.label(&format!("SCORE: {}", player_stats.score));
                ui.label(&format!(
                    "ENEMIES ALIVE: {}",
                    enemy_stats.current_enemy_amount
                ));
            });
        });

    let game_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 255),
        stroke: Stroke::new(5., Color32::WHITE),
        inner_margin: Margin {
            left: 15.,
            right: 15.,
            top: 15.,
            bottom: 15.,
        },
        ..default()
    };
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

fn pause_ui(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
) {
    let wnd = windows.get_primary().unwrap();

    let my_frame = Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 0),
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
                    ui.vertical_centered_justified(|ui| {
                        ui.label(&format!("PAUSED"));
                    });
                });
                ui.spacing_mut().item_spacing.y = 32.;

                ui.group(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        let mut style = ui.style_mut();
                        ui.spacing_mut().item_spacing.y = 8.;

                        if ui.button("Resume").clicked() {
                            commands.insert_resource(NextState(GameState::Playing));
                        }
                        if ui.button("Quit").clicked() {}
                    });
                });
                ui.columns(2, |ui| {
                    if ui[0].button("Resume").clicked() {
                        commands.insert_resource(NextState(GameState::Playing));
                    }
                    if ui[1].button("Quit").clicked() {
                        quit_game(exit);
                    }
                });
            });
        });
}

fn quit_game(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}
