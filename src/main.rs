mod game_systems;
mod player;
mod helpers;
mod ui;
mod enemy;
mod sound;

use crate::game_systems::*;
use crate::player::*;
use crate::ui::*;
use crate::enemy::EnemyPlugin;
use crate::sound::SoundPlugin;

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{close_on_esc, WindowMode};
use bevy_egui::*;
use bevy_rapier2d::prelude::*;



fn main() {
    App::new()
        .add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::GameSetupOnce)
                .with_collection::<AssetHolder>(),
        )

        .add_enter_system(GameState::GameSetupOnce, leave_game_setup_state)
        .add_enter_system(GameState::MainMenu, send_restart_game_event)
        .add_event::<RestartGameEvent>()
        //default resources needed
        .insert_resource(ClearColor(Color::rgba(0.05, 0.05, 0.1, 1.0)))
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.0))
        .add_plugin(EguiPlugin)
        .add_plugin(RapierDebugRenderPlugin::default())
        //
        //game base plugins
        .add_plugin(GameSystems)
        .add_plugin(PlayerPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(SoundPlugin)
        //
        //temp testing plugins
        //.add_system(close_on_esc)
        //
        .add_enter_system(GameState::Playing, turn_on_physics)
        .add_exit_system(GameState::Playing, turn_off_physics)
        .run();
}

pub(crate) struct RestartGameEvent;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    GameSetupOnce,
    MainMenu,
    Tutorial,
    Playing,
    Lose,
    Pause,
}

#[derive(AssetCollection)]
struct AssetHolder {
    #[asset(path = "player_planet.png")]
    pub player_planet: Handle<Image>,
    #[asset(path = "destroyed_planet.png")]
    pub player_planet_destroyed: Handle<Image>,
    #[asset(path = "player_missile.png")]
    pub player_missile: Handle<Image>,
    #[asset(path = "player_missile_explosion.png")]
    pub player_missile_explosion: Handle<Image>,
    #[asset(path = "player_missile_target.png")]
    pub player_missile_target: Handle<Image>,
    #[asset(path = "space_commander_logo.png")]
    pub logo: Handle<Image>,

    #[asset(path = "enemy.png")]
    pub enemy: Handle<Image>,
    #[asset(path = "enemy_ghost.png")]
    pub enemy_ghost: Handle<Image>,

    #[asset(path = "warning_sprite.png")]
    pub warning: Handle<Image>,

    #[asset(path = "OpenSans-ExtraBold.ttf")]
    pub font: Handle<Font>,
    /*
    #[asset(path = "music.ogg")]
    pub music: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "victory.ogg")]
    pub victory: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "death.ogg")]
    pub death: Handle<bevy_kira_audio::prelude::AudioSource>,

    #[asset(path = "apophis.png")]
    pub apophis: Handle<Image>,
    #[asset(path = "background.png")]
    pub background: Handle<Image>,
    */
}

fn leave_game_setup_state(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::MainMenu));
}

fn turn_on_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = true;
}

fn turn_off_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = false;
}

fn send_restart_game_event(mut event_writer: EventWriter<RestartGameEvent>) {
    event_writer.send(RestartGameEvent);
}
