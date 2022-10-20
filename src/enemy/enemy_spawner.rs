use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use crate::enemy::enemy_difficulty::EnemyStats;
use crate::enemy::{Enemy, GhostEnemyBundle, handle_enemy_collision_changes, handle_enemy_scanned, VisibilityTimer};
use crate::{AssetHolder, GameState};

pub(crate) struct EnemySpawnerPlugin;

impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        let mut fixed_update = SystemStage::parallel();

        fixed_update.add_system(
            spawn_next_wave
                // only do it in-game
                .run_in_state(GameState::Playing),
        );
        app.init_resource::<SpawnRes>()
            .add_event::<NewSpawnEvent>()
            .add_enter_system(GameState::MainMenu, setup_spawn_res)
            .add_enter_system(GameState::Playing, setup_warning_sprites);
        app.add_stage_before(
            CoreStage::Update,
            "FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_secs_f32(5.0), fixed_update),
        );

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("main_spawn_misc_loop")
                //.after("missile_post")
                .with_system(handle_spawn_events.run_on_event::<NewSpawnEvent>())
                .with_system(handle_visibility_timers)
                .into(),
        );
    }
}

// we need to get the window height and then use that to calculate how far left, right, top, and down are the out of bounds zones
//using that we can then have a function that returns a new random spawn spot based on that info in the resource
#[derive(Component, Copy, Clone, PartialEq, Eq)]
enum SpawnSide {
    Left,
    Top,
    Right,
    Bottom,
}

impl SpawnSide {
    fn spawn_warning_object(
        &self,
        sprites: &Res<AssetHolder>,
        spawn_res: &Res<SpawnRes>,
        mut commands: &mut Commands,
    ) {
        let mut new_spawn_location = Vec2 { x: 0.0, y: 0.0 };
        let mut spawn_side = SpawnSide::Left;
        match self {
            SpawnSide::Left => {
                new_spawn_location.x = spawn_res.left + 330.;
                spawn_side = SpawnSide::Left;
            }
            SpawnSide::Top => {
                new_spawn_location.y = spawn_res.top - 340.;
                spawn_side = SpawnSide::Top;
            }
            SpawnSide::Right => {
                new_spawn_location.x = spawn_res.right - 330.;
                spawn_side = SpawnSide::Right;
            }
            SpawnSide::Bottom => {
                new_spawn_location.y = spawn_res.bottom + 340.;
                spawn_side = SpawnSide::Bottom;
            }
        }
        commands.spawn_bundle(WarningBundle::new(sprites, new_spawn_location, spawn_side));
    }
}

struct NewSpawnEvent(SpawnSide);

struct SpawnRes {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,

    space: f32,
}

impl FromWorld for SpawnRes {
    fn from_world(_world: &mut World) -> Self {
        SpawnRes {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,

            space: 10.0,
        }
    }
}

impl SpawnRes {
    fn new_spawn_point(&self) -> (Vec2, SpawnSide) {
        let mut rng = thread_rng();
        let random = rng.gen_range(0..5);
        let mut new_spawn_location = Vec2 { x: 0.0, y: 0.0 };
        let mut spawn_side = SpawnSide::Left;
        match random {
            1 => {
                // left
                info!("LEFT");
                let range = rng.gen_range(self.bottom..self.top);
                new_spawn_location = Vec2 {
                    x: self.left - 50.,
                    y: range,
                };
                spawn_side = SpawnSide::Left;
            }
            2 => {
                // top
                info!("TOP");
                let range = rng.gen_range(self.left..self.right);
                new_spawn_location = Vec2 {
                    x: range,
                    y: self.top + 50.,
                };
                spawn_side = SpawnSide::Top;
            }
            3 => {
                // right
                info!("RIGHT");
                let range = rng.gen_range(self.bottom..self.top);
                new_spawn_location = Vec2 {
                    x: self.right + 50.,
                    y: range,
                };
                spawn_side = SpawnSide::Right;
            }
            _ => {
                // bottom and all else
                info!("BOTTOM");
                let range = rng.gen_range(self.left..self.right);
                new_spawn_location = Vec2 {
                    x: range,
                    y: self.bottom - 50.,
                };
                spawn_side = SpawnSide::Bottom;
            }
        }

        (new_spawn_location, spawn_side)
    }
}

#[derive(Component)]
struct Warning;

#[derive(Bundle)]
pub struct WarningBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    spawn_side: SpawnSide,
    warning: Warning,
    visibility_timer: VisibilityTimer,
}

impl WarningBundle {
    fn new(sprites: &Res<AssetHolder>, location: Vec2, spawn_side: SpawnSide) -> WarningBundle {
        WarningBundle {
            sprite_bundle: SpriteBundle {
                sprite: Default::default(),
                transform: Transform {
                    translation: location.extend(0.0),
                    scale: Vec3 {
                        x: 3.0,
                        y: 3.0,
                        z: 1.0,
                    },
                    ..default()
                },
                texture: sprites.warning.clone(),
                ..default()
            },
            spawn_side,
            warning: Warning,
            visibility_timer: VisibilityTimer {
                visibility_timer: Timer::new(Duration::from_secs_f32(0.5), false),
            },
        }
    }
}

fn setup_spawn_res(mut spawn_res: ResMut<SpawnRes>, windows: Res<Windows>) {
    let wnd = windows.get_primary().unwrap();
    let virtual_gameplay_size = Vec2::new(wnd.height() as f32, wnd.height() as f32);
    spawn_res.left = -(virtual_gameplay_size.x / 2. + spawn_res.space);
    spawn_res.top = virtual_gameplay_size.x / 2. + spawn_res.space;
    spawn_res.right = virtual_gameplay_size.x / 2. + spawn_res.space;
    spawn_res.bottom = -(virtual_gameplay_size.x / 2. + spawn_res.space);
}

fn setup_warning_sprites(
    sprites: Res<AssetHolder>,
    spawn_res: Res<SpawnRes>,
    mut commands: Commands,
) {
    SpawnSide::spawn_warning_object(&SpawnSide::Left, &sprites, &spawn_res, &mut commands);
    SpawnSide::spawn_warning_object(&SpawnSide::Top, &sprites, &spawn_res, &mut commands);
    SpawnSide::spawn_warning_object(&SpawnSide::Right, &sprites, &spawn_res, &mut commands);
    SpawnSide::spawn_warning_object(&SpawnSide::Bottom, &sprites, &spawn_res, &mut commands);
}

fn spawn_next_wave(
    sprites: Res<AssetHolder>,
    spawn_res: Res<SpawnRes>,
    enemy_stats: Res<EnemyStats>,
    mut commands: Commands,
    mut spawn_event_writer: EventWriter<NewSpawnEvent>,
) {
    for i in 0..enemy_stats.amount_to_spawn_a_wave {
        let (new_spawn_point, spawn_side) = &spawn_res.new_spawn_point();
        Enemy::spawn(&sprites, &enemy_stats, &mut commands, new_spawn_point);
        spawn_event_writer.send(NewSpawnEvent(*spawn_side));
    }
}

fn handle_spawn_events(
    mut spawn_event_reader: EventReader<NewSpawnEvent>,
    mut timed_enemies: Query<
        (Entity, &mut VisibilityTimer, &mut Visibility, &SpawnSide),
        With<Warning>,
    >,
) {
    for event in spawn_event_reader.iter() {
        for (entity, mut visibility_timer, mut visibility, spawn_side) in timed_enemies.iter_mut() {
            if *spawn_side == event.0 {
                *visibility = Visibility { is_visible: true };
                visibility_timer.visibility_timer.reset();    //: Timer::new(Duration::from_secs_f32(0.5), false);
            }
        }
    }
}

fn handle_visibility_timers(
    mut timed_enemies: Query<(Entity, &mut VisibilityTimer, &mut Visibility), With<Warning>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut visibility_timer, mut visibility) in timed_enemies.iter_mut() {
        visibility_timer.visibility_timer.tick(time.delta());
        if visibility_timer.visibility_timer.finished() {
            *visibility = Visibility { is_visible: false };
            //commands.entity(entity).remove::<VisibilityTimer>();
        }
    }
}
