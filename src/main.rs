mod player;
mod game_systems;

use crate::player::*;
use crate::game_systems::*;
use bevy::prelude::*;
use bevy::window::{close_on_esc, WindowMode};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
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

        //game base plugins
        .add_plugin(PlayerPlugin)
        .add_plugin(GameSystems)

        //temp testing plugins
        .add_system(close_on_esc)

        .run();
}
