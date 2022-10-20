use bevy::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;
use crate::GameState;

pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::GameSetupOnce, setup_game_systems);
    }
}

fn setup_game_systems(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
