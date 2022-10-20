use crate::player::input::input_manager::*;
use crate::player::*;
use crate::AssetHolder;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::enemy::{Destroyed, Enemy};

pub struct PlayerMissilePlugin;

impl Plugin for PlayerMissilePlugin {
    fn build(&self, app: &mut App) {
        app
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
pub struct SpawnMissileEvent {
    pub(crate) target: Vec2,
}

#[derive(Component)]
pub(crate) struct PlayerMissile {
    target: Vec2,
    reached_target: bool,
    time_since_explsion: f32,
    target_entity: Entity,
}

impl PlayerMissile {
    pub(crate) fn spawn(
        sprites: &Res<AssetHolder>,
        player_stats: &mut ResMut<PlayerStats>,
        commands: &mut Commands,
        mouse_pos: Vec2,
    ) {
        if player_stats.check_if_enough_energy(player_stats.missile_energy_cost) {
            player_stats.missile_fired();

            let target = mouse_pos;
            let angle = f32::atan2(mouse_pos.y - 0., mouse_pos.x - 0.);

            let missile_rotation = Quat::from_rotation_z(angle);
            let rotated_velocity = missile_rotation
                * Vec3 {
                x: player_stats.missile_speed,
                y: 0.0,
                z: 0.0,
            };
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
                PlayerMissile::spawn(&sprites, &mut player_stats, &mut commands, *target);
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
) {
    for (entity, transform, mut velocity, mut player_missile) in missile_query.iter_mut() {
        if player_missile.reached_target {
            player_missile.time_since_explsion += time.delta_seconds();
            if player_missile.time_since_explsion >= 0.2 {
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
        (Entity, &mut Handle<Image>, &PlayerMissile, &mut Velocity),
        Changed<PlayerMissile>,
    >,
    mut commands: Commands,
) {
    for (entity, mut sprite, player_missile, mut velocity) in missile_query.iter_mut() {
        if player_missile.reached_target {
            velocity.linvel = Vec2::ZERO;
            *sprite = sprites.player_missile_explosion.clone();
            commands.entity(entity).insert(Collider::ball(8.));
        }
    }
}

pub(crate) fn handle_missile_collisions(
    mut missiles: Query<(&CollidingEntities, &mut PlayerMissile), With<PlayerMissile>>,
    mut enemy_entities: Query<&Enemy>,
    mut commands: Commands,
) {
    for (entities, mut missiles) in missiles.iter_mut() {
        for collision in entities.iter() {
            if let Ok(_enemy) = enemy_entities.get(collision) {
                missiles.reached_target = true;
                commands.entity(collision).insert(Destroyed);
            }
        }
    }
}
