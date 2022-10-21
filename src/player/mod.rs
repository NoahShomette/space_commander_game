pub(crate) mod input;
mod player_missiles;
mod scanner;
mod shield;

use crate::player::player_missiles::player_missile_core::*;
use crate::player::scanner::scanner_core::*;
use crate::{AssetHolder, GameState};
use crate::player::input::input_manager::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use crate::enemy::{Destroyed, Enemy};
use crate::player::shield::shield_core::ShieldPlugin;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ScannerPlugin)
            .add_enter_system(GameState::GameSetupOnce, setup_player)
            //.add_exit_system(GameState::Playing, setup_player) //use this to rs
            .init_resource::<PlayerStats>()
            //main player loop
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .label("main_player_loop")
                    .with_system(handle_player_energy_recharge)
                    .with_system(handle_player_planet_collisions)
                    .into(),
            )
            .add_plugin(PlayerInputPlugin)
            .add_plugin(ShieldPlugin)
            .add_plugin(PlayerMissilePlugin);
    }
}

pub struct PlayerStats {
    pub(crate) is_regaining_energy: bool,
    pub(crate) max_energy: u32,
    pub(crate) current_energy: u32,
    pub(crate) energy_recharge_rate: f32,
    pub(crate) time_till_next_energy: f32,
    pub(crate) energy_per_recharge: u32,

    pub(crate) missile_speed: f32,
    pub(crate) missile_explosion_radius: f32,
    pub(crate) missile_energy_cost: u32,

    pub(crate) current_points: u32,
    pub(crate) score: u32,

    pub(crate) scan_speed: f32,
    pub(crate) scan_energy_cost: u32,

    pub(crate) shield_energy_cost: u32,
    pub(crate) shield_cost_rate: f32,

}


impl FromWorld for PlayerStats {
    fn from_world(_world: &mut World) -> Self {
        PlayerStats {
            is_regaining_energy: true,
            max_energy: 6,
            current_energy: 6,
            energy_recharge_rate: 2.0,
            time_till_next_energy: 0.,
            energy_per_recharge: 1,

            missile_speed: 200.,
            missile_explosion_radius: 0.0,
            missile_energy_cost: 1,
            current_points: 0,
            score: 0,

            scan_speed: 100.0,
            scan_energy_cost: 2,

            shield_energy_cost: 1,
            shield_cost_rate: 1.0,
        }
    }
}

impl PlayerStats {
    pub(crate) fn toggle_is_regaining_energy(&mut self) {
        self.is_regaining_energy = !self.is_regaining_energy;
    }

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

    pub(crate) fn scanner_fired(&mut self) {
        self.current_energy -= self.scan_energy_cost;
    }

    pub(crate) fn shield_cost(&mut self) {
        self.current_energy -= self.shield_energy_cost;
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

pub fn handle_player_energy_recharge(mut player_stats: ResMut<PlayerStats>, time: Res<Time>) {
    if player_stats.current_energy < player_stats.max_energy && player_stats.is_regaining_energy {
        player_stats.time_till_next_energy += time.delta_seconds();
        if player_stats.time_till_next_energy >= player_stats.energy_recharge_rate {
            player_stats.time_till_next_energy = 0.;
            player_stats.recharge_energy();
            info!("{}", player_stats.current_energy)
        }
    }
}

pub(crate) fn handle_player_planet_collisions(
    mut missiles: Query<(&CollidingEntities), With<Player>>,
    mut enemy_entities: Query<&Enemy>,
    mut commands: Commands,
) {
    for entities in missiles.iter_mut() {
        for collision in entities.iter() {
            if let Ok(_enemy) = enemy_entities.get(collision) {
                commands.insert_resource(NextState(GameState::MainMenu));
            }
        }
    }
}
