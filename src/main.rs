mod game_systems;
mod player;

use crate::game_systems::*;
use crate::player::*;
use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{close_on_esc, WindowMode};

fn main() {
    App::new()
        //default resources needed
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            title: "Space Commander".to_string(),
            resizable: false,
            decorations: false,
            cursor_visible: true,
            cursor_locked: true,
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        // base plugins
        .add_plugins(DefaultPlugins)
        //
        //game base plugins
        .add_plugin(GameSystems)
        .add_plugin(PlayerPlugin)
        //
        //temp testing plugins
        .add_system(close_on_esc)
        //
        .run();
}
