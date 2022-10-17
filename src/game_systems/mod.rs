use bevy::prelude::*;

pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_game_systems);
    }
}

fn setup_game_systems(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
