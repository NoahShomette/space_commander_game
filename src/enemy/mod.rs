pub mod enemy_difficulty;
pub mod enemy_spawner;

use crate::enemy::enemy_difficulty::{EnemyDifficultyPlugin, EnemyStats, update_enemy_count};
use crate::enemy::enemy_spawner::EnemySpawnerPlugin;
use crate::{AssetHolder, GameState, PlayerStats, RestartGameEvent, ScoreEvent};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

pub(crate) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EnemySpawnerPlugin);
        app.add_plugin(EnemyDifficultyPlugin);
        app.add_system_set(
            ConditionSet::new()
                .with_system(handle_restart_game_events.run_on_event::<RestartGameEvent>())
                .into(),
        );

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("main_enemy_loop")
                .after("missile_post")
                .with_system(handle_enemy_scanned)
                .with_system(handle_visibility_timers)
                .into(),
        );
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("post_enemy_loop")
                .after("main_enemy_loop")
                .with_system(handle_enemy_collision_changes)
                .with_system(update_enemy_count)
                .into(),
        );
    }
}

#[derive(Component)]
pub(crate) struct Enemy {
    pub(crate) scan_ghost: Entity,
}

#[derive(Component)]
pub(crate) struct VisibilityTimer {
    visibility_timer: Timer,
}

impl Enemy {
    pub(crate) fn spawn(
        sprites: &Res<AssetHolder>,
        enemy_stats: &ResMut<EnemyStats>,
        mut commands: &mut Commands,
        spawn_location: &Vec2,
    ) {
        let spawn_location_local = spawn_location.clone();
        let angle = f32::atan2(-spawn_location_local.y, -spawn_location_local.x);

        let missile_rotation = Quat::from_rotation_z(angle);
        let rotated_velocity = missile_rotation
            * Vec3 {
            x: enemy_stats.speed.clone(),
            y: 0.0,
            z: 0.0,
        };
        let ghost_entity = commands
            .spawn_bundle(GhostEnemyBundle::new(
                &sprites,
                spawn_location_local.clone(),
            ))
            .id();
        commands.spawn_bundle(EnemyBundle::new(
            &sprites,
            &spawn_location_local,
            rotated_velocity.truncate(),
            missile_rotation,
            ghost_entity,
        ));
    }
}

#[derive(Component)]
pub(crate) struct Destroyed;

#[derive(Component)]
pub(crate) struct ReachedPlanet;

#[derive(Component)]
pub(crate) struct Scanned;

#[derive(Bundle)]
pub struct EnemyBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    rigidbody: RigidBody,
    collider: Collider,
    enemy: Enemy,
    gravity_scale: GravityScale,
    velocity: Velocity,
    sensor: Sensor,
    ccd: Ccd,
}

impl EnemyBundle {
    pub(crate) fn new(
        sprites: &Res<AssetHolder>,
        spawn_location: &Vec2,
        linvel: Vec2,
        rotation: Quat,
        ghost_entity: Entity,
    ) -> EnemyBundle {
        EnemyBundle {
            sprite_bundle: SpriteBundle {
                sprite: Default::default(),
                transform: Transform {
                    translation: spawn_location.extend(10.),
                    rotation,
                    scale: Vec3 {
                        x: 3.0,
                        y: 3.0,
                        z: 1.0,
                    },
                },
                global_transform: Default::default(),
                texture: sprites.enemy.clone(),

                visibility: Visibility { is_visible: false },
                computed_visibility: Default::default(),
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::ball(5.),
            gravity_scale: GravityScale(0.),
            enemy: Enemy {
                scan_ghost: ghost_entity,
            },
            velocity: Velocity {
                linvel,
                angvel: 0.0,
            },
            sensor: Sensor,
            ccd: Ccd::enabled(),
        }
    }
}

#[derive(Bundle)]
pub struct GhostEnemyBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    pub(crate) ghost: Ghost,
    pub(crate) collider: Collider,
    rigidbody: RigidBody,
    gravity_scale: GravityScale,
    sensor: Sensor,
}

#[derive(Component)]
pub struct Ghost(bool);

impl GhostEnemyBundle {
    pub(crate) fn new(sprites: &Res<AssetHolder>, scanned_location: Vec2) -> GhostEnemyBundle {
        GhostEnemyBundle {
            sprite_bundle: SpriteBundle {
                sprite: Default::default(),
                transform: Transform {
                    translation: scanned_location.extend(0.0),
                    scale: Vec3 {
                        x: 3.0,
                        y: 3.0,
                        z: 1.0,
                    },
                    ..default()
                },
                texture: sprites.enemy_ghost.clone(),
                ..default()
            },
            ghost: Ghost(false),
            collider: Collider::ball(5.),
            rigidbody: RigidBody::Dynamic,
            gravity_scale: GravityScale(0.),
            sensor: Sensor,
        }
    }
}

pub(crate) fn handle_enemy_collision_changes(
    mut destroyed_enemies: Query<(Entity, &Enemy), With<Destroyed>>,
    mut ghost_query: Query<&mut Ghost>,
    mut commands: Commands,
    mut score_event_writer: EventWriter<ScoreEvent>,
    player_stats: Res<PlayerStats>,
) {
    for (destroyed_enemy, enemy) in destroyed_enemies.iter_mut() {
        if let Ok(mut ghost) = ghost_query.get_mut(enemy.scan_ghost) {
            ghost.0 = true;
        }
        commands.entity(destroyed_enemy).despawn();
        score_event_writer.send(ScoreEvent(player_stats.enemy_kill_score));
    }
}

pub(crate) fn handle_enemy_scanned(
    mut scanned_enemies: Query<
        (
            Entity,
            Option<&Enemy>,
            Option<&Ghost>,
            &Transform,
            &mut Visibility,
        ),
        With<Scanned>,
    >,
    mut ghost_query: Query<&mut Transform, Without<Scanned>>,
    mut commands: Commands,
) {
    for (scanned_enemy, enemy_option, ghost_option, transform, mut visibility) in
    scanned_enemies.iter_mut()
    {
        //handles
        if enemy_option.is_some() {
            let enemy = enemy_option.unwrap();
            if let Ok(mut ghost_transform) = ghost_query.get_mut(enemy.scan_ghost) {
                ghost_transform.translation = transform.translation;
            }
            *visibility = Visibility { is_visible: true };
            commands.entity(scanned_enemy).insert(VisibilityTimer {
                visibility_timer: Timer::new(Duration::from_secs_f32(1.0), false),
            });
            commands.entity(scanned_enemy).remove::<Scanned>();
        }

        if ghost_option.is_some() {
            let ghost = ghost_option.unwrap();
            if ghost.0 == true {
                commands.entity(scanned_enemy).despawn();
            } else {
                commands.entity(scanned_enemy).remove::<Scanned>();
            }
        }
    }
}

pub(crate) fn handle_visibility_timers(
    mut timed_enemies: Query<(Entity, &mut VisibilityTimer, &mut Visibility), With<Enemy>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut visibility_timer, mut visibility) in timed_enemies.iter_mut() {
        visibility_timer.visibility_timer.tick(time.delta());
        if visibility_timer.visibility_timer.finished() {
            info!("visibility timer finished");
            *visibility = Visibility { is_visible: false };
            commands.entity(entity).remove::<VisibilityTimer>();
        }
    }
}


fn handle_restart_game_events(
    mut commands: Commands,
    mut enemies: Query<Entity, With<Enemy>>,
    mut ghosts: Query<Entity, With<Ghost>>,
) {
    for enemy in enemies.iter_mut() {
        commands.entity(enemy).despawn();
    }
    for enemy in ghosts.iter_mut() {
        commands.entity(enemy).despawn();
    }
    commands.insert_resource(EnemyStats::default());
}
