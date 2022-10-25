mod enemy;
mod game_systems;
mod helpers;
mod player;
mod sound;
mod ui;

use crate::enemy::EnemyPlugin;
use crate::game_systems::*;
use crate::player::*;
use crate::sound::SoundPlugin;
use crate::ui::*;
use bevy::asset::AssetServerSettings;

use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{close_on_esc, PresentMode, WindowMode, WindowResizeConstraints};
use bevy_asset_loader::prelude::*;
use bevy_egui::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

fn main() {
    App::new()
        .add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::GameSetupOnce)
                .with_collection::<AssetHolder>()
                .with_collection::<SoundAssetHolder>(),
        )
        .add_enter_system(GameState::GameSetupOnce, leave_game_setup_state)
        .add_enter_system(GameState::MainMenu, send_restart_game_event)
        .add_event::<RestartGameEvent>()
        .init_resource::<GameSettings>()
        //default resources needed
        .insert_resource(ClearColor(Color::rgba(0.05, 0.05, 0.1, 1.0)))
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            title: "Space Commander".to_string(),
            width: 1920.0,
            height: 1080.0,
            position: WindowPosition::Automatic,
            resize_constraints: WindowResizeConstraints {
                min_width: 960.0,
                min_height: 480.0,
                ..Default::default()
            },
            scale_factor_override: Some(1.0), //Needed for some mobile devices, but disables scaling
            present_mode: PresentMode::AutoVsync,
            resizable: true,
            decorations: true,
            cursor_locked: false,
            cursor_visible: true,
            mode: WindowMode::Windowed,
            transparent: false,
            canvas: Some("#bevy".to_string()),
            fit_canvas_to_parent: true,
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        // base plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.0))
        .add_plugin(EguiPlugin)
        .add_plugin(AudioPlugin)
        //.add_plugin(RapierDebugRenderPlugin::default())
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

pub(crate) struct GameSettings {
    is_sound_on: bool,
    sound_level: (f32, f32, f32),
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            is_sound_on: true,
            sound_level: (0.0, 75.0, 100.0),
        }
    }
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
    #[asset(path = "player_missile_explosion_large.png")]
    pub player_missile_explosion_large: Handle<Image>,
    #[asset(path = "player_missile_explosion_medium.png")]
    pub player_missile_explosion_medium: Handle<Image>,
    #[asset(path = "player_missile_target.png")]
    pub player_missile_target: Handle<Image>,

    #[asset(path = "space_commander_logo.png")]
    pub logo: Handle<Image>,
    #[asset(path = "player_health.png")]
    pub health: Handle<Image>,
    #[asset(path = "player_health_empty.png")]
    pub health_empty: Handle<Image>,
    #[asset(path = "black_bg.png")]
    pub bg: Handle<Image>,

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

#[derive(AssetCollection)]
struct SoundAssetHolder {
    #[asset(path = "sounds/648411__davejf__melody-loop-130-bpm.mp3")]
    pub music: Handle<bevy_kira_audio::prelude::AudioSource>,

    #[asset(path = "sounds/70299__kizilsungur__sonar.wav")]
    pub scan_launch: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "sounds/543932__mrlindstrom__sonarping.wav")]
    pub scan_ping: Handle<bevy_kira_audio::prelude::AudioSource>,

    #[asset(path = "sounds/653813__eilassfx__error.wav")]
    pub enemy_warning: Handle<bevy_kira_audio::prelude::AudioSource>,

    #[asset(path = "sounds/135125__ecfike__computer-error.wav")]
    pub normal_button: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "sounds/404021__deathscyp__error.wav")]
    pub error_button: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "sounds/541998__rob-marion__gasp-power-up.wav")]
    pub super_upgrade: Handle<bevy_kira_audio::prelude::AudioSource>,

    #[asset(path = "sounds/523224__bbqgiraffe__distant-explosion.wav")]
    pub missile_explosion: Handle<bevy_kira_audio::prelude::AudioSource>,
    //TODO UPDATE TO LAUNCH SOUND
    #[asset(path = "sounds/458664__jorgerosa__missile-explosion.ogg")]
    pub missile_launch: Handle<bevy_kira_audio::prelude::AudioSource>,

    #[asset(path = "sounds/351450__cabled-mess__shield-energy-loop-01.wav")]
    pub shield_on: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "sounds/257565__udderdude__bfxr2.wav")]
    pub shield_hit: Handle<bevy_kira_audio::prelude::AudioSource>,

    #[asset(path = "sounds/649191__ayadrevis__explosion.ogg")]
    pub planet_damage: Handle<bevy_kira_audio::prelude::AudioSource>,
    /*
    #[asset(path = "music.ogg")]
    pub music: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "victory.ogg")]
    pub victory: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "death.ogg")]
    pub death: Handle<bevy_kira_audio::prelude::AudioSource>,
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
