use crate::player::*;
use crate::AssetHolder;
use bevy::prelude::*;
use bevy_rapier2d::na::RealField;
use bevy_rapier2d::prelude::*;

const PLAYER_MISSILE_ASSET_PATH: &str = "player_missile.png";

#[derive(Default)]
pub struct SpawnMissileEvent {
    pub(crate) target: Vec2,
}

#[derive(Component)]
pub(crate) struct PlayerMissile {
    target: Vec2,
    reached_target: bool,
    time_since_explsion: f32,
}

impl PlayerMissile {
    pub(crate) fn explode(&self) {}

    pub(crate) fn spawn(
        asset_server: &Res<AssetServer>,
        mut player_stats: &mut ResMut<PlayerStats>,
        mut commands: &mut Commands,
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
            commands.spawn_bundle(PlayerMissileBundle::new(
                asset_server,
                rotated_velocity.truncate(),
                missile_rotation,
                target,
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
    collider: Collider,
    ccd: Ccd,
    gravity_scale: GravityScale,
    active_events: ActiveEvents,
    colliding_entities: CollidingEntities,
    locked_axes: LockedAxes,

    player_missile: PlayerMissile,
}

impl PlayerMissileBundle {
    pub(crate) fn new(
        asset_server: &Res<AssetServer>,
        linvel: Vec2,
        rotation: Quat,
        target: Vec2,
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
                texture: asset_server.load(PLAYER_MISSILE_ASSET_PATH),
                ..default()
            },
            velocity: Velocity {
                linvel,
                angvel: 0.0,
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::ball(1.),
            gravity_scale: GravityScale(0.),
            active_events: ActiveEvents::COLLISION_EVENTS,
            colliding_entities: Default::default(),
            ccd: Ccd::enabled(),
            locked_axes: LockedAxes::all(),
            player_missile: PlayerMissile {
                target,
                reached_target: false,
                time_since_explsion: 0.0,
            },
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
    mut missile_query: Query<(&mut Handle<Image>, &mut Collider, &PlayerMissile), Changed<PlayerMissile>>,
) {
    for (mut sprite, mut collider, player_missile) in missile_query.iter_mut() {
        if player_missile.reached_target {
            *sprite = sprites.player_missile_explosion.clone();
            *collider = Collider::ball(8.);
        }
    }
}
