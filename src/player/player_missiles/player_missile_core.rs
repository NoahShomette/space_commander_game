use crate::player::input::input_manager::*;
use crate::player::*;
use crate::AssetHolder;

use bevy::prelude::*;
use bevy_rapier2d::parry::transformation::utils::transform;
use bevy_rapier2d::prelude::*;
use crate::enemy::{Destroyed, Enemy};

pub struct PlayerMissilePlugin;

impl Plugin for PlayerMissilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EnemyKilledEvent>()
            //handles spawning missiles events and updating missiles/checking if they have arrived
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .label("missile_main")
                    .after("player_input")
                    .with_system(
                        handle_player_missile_spawn_events.run_on_event::<PlayerInputEvents>(),
                    )
                    .with_system(update_missiles)
                    .into(),
            );

        app.add_system_set(
            ConditionSet::new()
                .with_system(handle_restart_game_events.run_on_event::<RestartGameEvent>())
                .into()
        );
        //handles missiles exploding
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("missile_post")
                .before("missile_main")
                .with_system(missile_explode)
                .with_system(handle_missile_collisions)
                .into(),
        );
    }
}

#[derive(Default)]
pub struct EnemyKilledEvent {
    pub(crate) location: Vec2,
}

#[derive(Default)]
pub struct SpawnMissileEvent {
    pub(crate) target: Vec2,
}

#[derive(Component)]
pub(crate) struct PlayerMissile {
    target: Vec2,
    reached_target: bool,
    time_since_explsion: f32,
    target_entity: Entity,
    enemy_killed: bool,
}

impl PlayerMissile {
    pub(crate) fn spawn(
        sprites: &Res<AssetHolder>,
        player_stats: &mut ResMut<PlayerStats>,
        commands: &mut Commands,
        mouse_pos: Vec2,
        is_cluster_missile: bool,
    ) {
        if player_stats.check_if_enough_energy(player_stats.missile_energy_cost) || is_cluster_missile {
            if !is_cluster_missile {
                player_stats.missile_fired();
            }

            let target = mouse_pos;
            let angle = f32::atan2(mouse_pos.y - 0., mouse_pos.x - 0.);

            let missile_rotation = Quat::from_rotation_z(angle);
            let rotated_velocity = missile_rotation
                * Vec3 {
                x: player_stats.missile_speed.0,
                y: 0.0,
                z: 0.0,
            };
            let is_larger = player_stats.is_larger_missiles_upgrade;
            let missile_target = commands
                .spawn_bundle(PlayerMissileTargetBundle::new(sprites, target))
                .id();
            commands.spawn_bundle(PlayerMissileBundle::new(
                sprites,
                rotated_velocity.truncate(),
                missile_rotation,
                target,
                missile_target,
            ));
        }
    }
}

#[derive(Bundle)]
pub struct PlayerMissileBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    velocity: Velocity,
    rigidbody: RigidBody,
    ccd: Ccd,
    gravity_scale: GravityScale,
    active_events: ActiveEvents,
    colliding_entities: CollidingEntities,
    locked_axes: LockedAxes,

    player_missile: PlayerMissile,
}

#[derive(Bundle)]
pub struct PlayerMissileTargetBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
}

impl PlayerMissileTargetBundle {
    pub(crate) fn new(sprites: &Res<AssetHolder>, target: Vec2) -> PlayerMissileTargetBundle {

        PlayerMissileTargetBundle {
            sprite_bundle: SpriteBundle {
                sprite: Default::default(),
                transform: Transform {
                    translation: target.extend(0.0),
                    scale: Vec3 {
                        x: 3.0,
                        y: 3.0,
                        z: 1.0,
                    },
                    ..default()
                },
                texture: sprites.player_missile_target.clone(),
                ..default()
            },
        }
    }
}

impl PlayerMissileBundle {
    pub(crate) fn new(
        sprites: &Res<AssetHolder>,
        linvel: Vec2,
        rotation: Quat,
        target: Vec2,
        target_entity: Entity,
    ) -> PlayerMissileBundle {
        PlayerMissileBundle {
            sprite_bundle: SpriteBundle {
                sprite: Default::default(),
                transform: Transform {
                    translation: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 50.0,
                    },
                    rotation,
                    scale: Vec3 {
                        x: 3.0,
                        y: 3.0,
                        z: 1.0,
                    },
                },
                global_transform: Default::default(),
                texture: sprites.player_missile.clone(),
                ..default()
            },
            velocity: Velocity {
                linvel,
                angvel: 0.0,
            },
            rigidbody: RigidBody::Dynamic,
            gravity_scale: GravityScale(0.),
            active_events: ActiveEvents::COLLISION_EVENTS,
            colliding_entities: Default::default(),
            ccd: Ccd::enabled(),
            locked_axes: LockedAxes::all(),
            player_missile: PlayerMissile {
                target,
                reached_target: false,
                time_since_explsion: 0.0,
                target_entity,
                enemy_killed: false
            },
        }
    }
}

pub(crate) fn handle_player_missile_spawn_events(
    sprites: Res<AssetHolder>,
    mut player_stats: ResMut<PlayerStats>,
    mut commands: Commands,
    mut spawn_missile_event_reader: EventReader<PlayerInputEvents>,
) {
    for event in spawn_missile_event_reader.iter() {
        match event {
            PlayerInputEvents::FireMissile(target) => {
                PlayerMissile::spawn(&sprites, &mut player_stats, &mut commands, *target, false);
                if player_stats.is_cluster_missile_upgrade {
                    let mut cluster_dif: f32 = 20.;
                    if player_stats.is_larger_missiles_upgrade {
                        cluster_dif = 40.;
                    }
                    PlayerMissile::spawn(&sprites, &mut player_stats, &mut commands, Vec2 { x: target.x, y: target.y + cluster_dif }, true);
                    PlayerMissile::spawn(&sprites, &mut player_stats, &mut commands, Vec2 { x: target.x, y: target.y - cluster_dif }, true);
                    PlayerMissile::spawn(&sprites, &mut player_stats, &mut commands, Vec2 { x: target.x + cluster_dif, y: target.y }, true);
                    PlayerMissile::spawn(&sprites, &mut player_stats, &mut commands, Vec2 { x: target.x - cluster_dif, y: target.y }, true);
                }
            }
            PlayerInputEvents::Scan => {}
            PlayerInputEvents::Shield(_) => {}
        }
    }
}

pub(crate) fn update_missiles(
    mut missile_query: Query<(Entity, &GlobalTransform, &mut Velocity, &mut PlayerMissile)>,
    time: Res<Time>,
    mut commands: Commands,
    mut player_stats: ResMut<PlayerStats>,
) {
    for (entity, transform, mut velocity, mut player_missile) in missile_query.iter_mut() {
        if player_missile.reached_target {
            player_missile.time_since_explsion += time.delta_seconds();
            if player_missile.time_since_explsion >= 0.2 {
                if player_stats.is_energy_vampire_upgrade && player_missile.enemy_killed {
                    player_stats.plus_one_energy();
                }
                commands.entity(player_missile.target_entity).despawn();
                commands.entity(entity).despawn();
            }
        }

        let dif = transform.translation().truncate() - player_missile.target;

        let magnitude = dif.abs();

        if magnitude.x <= 5.0 && magnitude.y <= 5.0 {
            player_missile.reached_target = true;
            velocity.linvel = Vec2::ZERO;
        }
    }
}

pub(crate) fn missile_explode(
    sprites: Res<AssetHolder>,
    mut missile_query: Query<
        (Entity, &mut Handle<Image>, &PlayerMissile, &mut Velocity, &mut Transform),
        Changed<PlayerMissile>,
    >,
    mut commands: Commands,
    player_stats: Res<PlayerStats>,
) {
    for (entity, mut sprite, player_missile, mut velocity, mut transform) in missile_query.iter_mut() {
        if player_missile.reached_target {
            velocity.linvel = Vec2::ZERO;
            let mut radius: f32 = 8.;
            if player_stats.is_larger_missiles_upgrade {
                radius = 12.;
                *sprite = sprites.player_missile_explosion_medium.clone();
            } else {
                *sprite = sprites.player_missile_explosion.clone();
            }

            commands.entity(entity).insert(Collider::ball(radius));
        }
    }
}

pub(crate) fn handle_missile_collisions(
    mut missiles: Query<(&CollidingEntities, &mut PlayerMissile), With<PlayerMissile>>,
    mut enemy_entities: Query<(&Enemy, &Transform)>,
    mut commands: Commands,
    mut enemy_killed_event_writer: EventWriter<EnemyKilledEvent>,
) {
    for (entities, mut missiles) in missiles.iter_mut() {
        for collision in entities.iter() {
            if let Ok(_enemy) = enemy_entities.get(collision) {
                missiles.reached_target = true;
                missiles.enemy_killed = true;
                commands.entity(collision).insert(Destroyed);
                enemy_killed_event_writer.send(EnemyKilledEvent { location: _enemy.1.translation.truncate() });
            }
        }
    }
}

fn handle_restart_game_events(mut commands: Commands, mut missiles: Query<(Entity, &PlayerMissile)>) {
    for (missile, player_missile) in missiles.iter_mut() {
        commands.entity(player_missile.target_entity).despawn();
        commands.entity(missile).despawn();
    }
}
