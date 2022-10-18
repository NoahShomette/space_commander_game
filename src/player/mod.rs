mod player_missiles;
mod scanner;

use crate::helpers::*;
use crate::player::player_missiles::player_missile_core::*;
use crate::{AssetHolder, GameState};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, setup_player)
            .init_resource::<PlayerStats>()
            .add_event::<SpawnMissileEvent>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .label("main_player_loop")
                    .with_system(player_input)
                    .with_system(
                        handle_player_missile_spawn_events.run_on_event::<SpawnMissileEvent>(),
                    )
                    .with_system(update_missiles)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .after("main_player_loop")
                    .with_system(missile_explode)
                    .with_system(handle_player_energy_recharge)
                    .into(),
            );
    }
}

pub struct PlayerStats {
    max_energy: u32,
    current_energy: u32,
    energy_recharge_rate: f32,
    time_till_next_energy: f32,
    energy_per_recharge: u32,

    missile_speed: f32,
    missile_explosion_radius: f32,
    missile_energy_cost: u32,

    current_points: u32,
    pub(crate) score: u32,
}

impl PlayerStats {
    pub(crate) fn recharge_energy(&mut self) {
        self.current_energy += self.energy_per_recharge;
        if self.current_energy > self.max_energy {
            self.current_energy = self.max_energy;
        }
    }

    pub(crate) fn check_if_enough_energy(&self, amount_needed: u32) -> bool {
        return if self.current_energy >= amount_needed {
            true
        } else {
            false
        };
    }

    pub(crate) fn missile_fired(&mut self) {
        self.current_energy -= self.missile_energy_cost;
    }
}

impl FromWorld for PlayerStats {
    fn from_world(_world: &mut World) -> Self {
        PlayerStats {
            max_energy: 6,
            current_energy: 6,
            energy_recharge_rate: 4.0,
            time_till_next_energy: 4.0,
            energy_per_recharge: 1,

            missile_speed: 200.,
            missile_explosion_radius: 0.0,
            missile_energy_cost: 1,
            current_points: 0,
            score: 0,
        }
    }
}

fn setup_player(mut commands: Commands, sprites: Res<AssetHolder>) {
    commands.spawn_bundle(PlayerBundle::new(sprites));
}

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    rigidbody: RigidBody,
    collider: Collider,
    player: Player,
    gravity_scale: GravityScale,
    active_events: ActiveEvents,
    colliding_entities: CollidingEntities,
}

impl PlayerBundle {
    pub(crate) fn new(sprites: Res<AssetHolder>) -> PlayerBundle {
        PlayerBundle {
            sprite_bundle: SpriteBundle {
                sprite: Default::default(),
                transform: Transform {
                    translation: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 50.0,
                    },
                    rotation: Default::default(),
                    scale: Vec3 {
                        x: 3.0,
                        y: 3.0,
                        z: 1.0,
                    },
                },
                global_transform: Default::default(),
                texture: sprites.player_planet.clone(),
                ..default()
            },
            rigidbody: RigidBody::Fixed,
            collider: Collider::ball(8.),
            player: Player,
            gravity_scale: GravityScale(0.),
            active_events: ActiveEvents::COLLISION_EVENTS,
            colliding_entities: Default::default(),
        }
    }
}

pub fn player_input(
    keyboard_input: Res<Input<MouseButton>>,
    mut spawn_missile_event_writer: EventWriter<SpawnMissileEvent>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if keyboard_input.just_pressed(MouseButton::Left) {
        if mouse_virtual_play_field_check(&windows, &camera_query) {
            let mouse_world_pos = mouse_screen_pos_to_world_pos(windows, camera_query);
            spawn_missile_event_writer.send(SpawnMissileEvent {
                target: mouse_world_pos,
            });
        }
    }
}

pub fn handle_player_missile_spawn_events(
    asset_server: Res<AssetServer>,
    mut player_stats: ResMut<PlayerStats>,
    mut commands: Commands,
    mut spawn_missile_event_reader: EventReader<SpawnMissileEvent>,
) {
    for event in spawn_missile_event_reader.iter() {
        PlayerMissile::spawn(
            &asset_server,
            &mut player_stats,
            &mut commands,
            event.target,
        );
    }
}

pub fn handle_player_energy_recharge(mut player_stats: ResMut<PlayerStats>, time: Res<Time>) {
    if player_stats.current_energy < player_stats.max_energy {
        player_stats.time_till_next_energy -= time.delta_seconds();
        if player_stats.time_till_next_energy <= 0. {
            player_stats.time_till_next_energy = player_stats.energy_recharge_rate;
            player_stats.recharge_energy();
            info!("{}", player_stats.current_energy)
        }
    }
}
