use crate::egui::Context;
use crate::*;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget::Image;
use bevy_egui::egui::*;
use bevy_egui::*;
use iyes_loopless::prelude::*;

pub struct UiPlugin;

//const SELECTED_COLOR: Color32 = Color32::from_rgb(94 / 3, 255 / 3, 169 / 3);
//const DESELECTED_COLOR: Color32 = Color32::from_rgb(94 / 10, 255 / 10, 169 / 10);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::MainMenu)
                .with_system(outside_backgrounds)
                .into(),
        )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .with_system(outside_backgrounds)
                    .into(),
            );
    }
}



fn outside_backgrounds(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    sprites: Res<AssetHolder>,
    player_stats: Res<PlayerStats>,
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
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(&format!("{}:{}", player_stats.current_energy, player_stats.max_energy))
            });
        });

    SidePanel::right("right_background")
        .frame(my_frame)
        .resizable(false)
        .min_width((wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.))
        .max_width((wnd.width() as f32 / 2.) - (wnd.height() as f32 / 2.))
        .show(egui_context.ctx_mut(), |ui| {
            let mut style = ui.style_mut();
            style.visuals.override_text_color = Some(Color32::from_rgb(255, 255, 255));
            //style.visuals.widgets.active.bg_fill = DESELECTED_COLOR;

            ui.vertical_centered_justified(|ui| {
                ui.image(logo, [384.0, 192.0]);
                ui.label(&format!("SCORE: {}", player_stats.score))
            });
        });
}


/*

    let mut fonts = egui::FontDefinitions::default();
    // Large button text:
    fonts.families.get_mut(&FontFamily::Proportional).unwrap()
        .insert(0, "OpenSans-ExtraBold.ttf".to_owned());

    egui_context.ctx_mut().set_fonts(fonts);
 */