﻿pub(crate) mod input;
mod player_missiles;
mod scanner;
mod shield;

use crate::player::input::input_manager::*;
use crate::player::player_missiles::player_missile_core::*;
use crate::player::scanner::scanner_core::*;
use crate::{AssetHolder, GameState, RestartGameEvent};

use crate::enemy::{Destroyed, Enemy};
use crate::player::shield::shield_core::ShieldPlugin;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ScannerPlugin)
            .add_enter_system(GameState::GameSetupOnce, setup_player)
            .add_system_set(
                ConditionSet::new()
                    .with_system(handle_restart_game_events.run_on_event::<RestartGameEvent>())
                    .into(),
            )
            //.add_exit_system(GameState::Playing, setup_player) //use this to rs
            .init_resource::<PlayerStats>()
            .add_event::<ScoreEvent>()
            //main player loop
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .label("main_player_loop")
                    .with_system(handle_player_energy_recharge)
                    .with_system(handle_player_planet_collisions)
                    .with_system(handle_score_events.run_on_event::<ScoreEvent>())
                    .into(),
            )
            .add_plugin(PlayerInputPlugin)
            .add_plugin(ShieldPlugin)
            .add_plugin(PlayerMissilePlugin);
    }
}

pub struct PlayerStats {
    pub(crate) current_health: u32,
    pub(crate) max_health: u32,
    pub(crate) health_recharge_time: (f32, f32, f32),

    pub(crate) is_regaining_energy: bool,
    pub(crate) max_energy: u32,
    pub(crate) current_energy: u32,
    pub(crate) energy_recharge_rate: (f32, f32, f32),
    pub(crate) time_till_next_energy: f32,
    pub(crate) energy_per_recharge: u32,

    pub(crate) missile_speed: (f32, f32),
    pub(crate) missile_energy_cost: u32,

    pub(crate) current_points: u32,
    pub(crate) score: u32,
    pub(crate) locked_score: u32,

    pub(crate) scan_speed: (f32, f32, f32),
    pub(crate) scan_energy_cost: u32,

    pub(crate) shield_energy_cost: u32,
    pub(crate) shield_cost_rate: f32,

    pub(crate) enemy_kill_score: u32,

    //costs for upgrades
    pub(crate) max_energy_upgrade_cost: u32,
    pub(crate) energy_recharge_rate_upgrade_cost: u32,

    pub(crate) missile_energy_cost_upgrade_cost: u32,

    pub(crate) max_health_upgrade_cost: u32,
    pub(crate) current_health_increase_cost: u32,
    pub(crate) health_recharge_cost: u32,

    pub(crate) scan_speed_upgrade_cost: u32,

    pub(crate) shield_time_upgrade_cost: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats {
            current_health: 2,
            max_health: 2,
            health_recharge_time: (15., 1., 1.),

            is_regaining_energy: true,
            max_energy: 6,
            current_energy: 6,
            energy_recharge_rate: (4.0, 0.1, 0.3),
            time_till_next_energy: 0.,
            energy_per_recharge: 1,

            missile_speed: (100., 500.),
            missile_energy_cost: 1,
            current_points: 0,
            score: 4000000000,
            locked_score: 0,

            scan_speed: (50.0, 200., 25.),
            scan_energy_cost: 2,

            shield_energy_cost: 1,
            shield_cost_rate: 1.0,
            enemy_kill_score: 5,

            //costs for upgrades
            max_energy_upgrade_cost: 35,
            energy_recharge_rate_upgrade_cost: 25,

            missile_energy_cost_upgrade_cost: 0,

            max_health_upgrade_cost: 50,
            current_health_increase_cost: 30,
            health_recharge_cost: 30,

            scan_speed_upgrade_cost: 35,

            shield_time_upgrade_cost: 35,
        }
    }
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

    pub(crate) fn scanner_fired(&mut self) {
        self.current_energy -= self.scan_energy_cost;
    }

    pub(crate) fn shield_cost(&mut self) {
        self.current_energy -= self.shield_energy_cost;
    }

    //upgrades

    pub(crate) fn upgrade_max_energy(&mut self) {
        if self.check_if_enough_score(self.max_energy_upgrade_cost) {
            self.max_energy += 1;
            self.remove_score(self.max_energy_upgrade_cost);
        }
    }

    pub(crate) fn upgrade_energy_charge(&mut self) {
        if self.check_if_enough_score(self.energy_recharge_rate_upgrade_cost) {
            self.energy_per_recharge += 1;
            self.remove_score(self.energy_recharge_rate_upgrade_cost);
        }
    }

    pub(crate) fn upgrade_max_health(&mut self) {
        if self.check_if_enough_score(self.max_health_upgrade_cost) {
            self.max_health += 1;
            self.remove_score(self.max_health_upgrade_cost);
        }
    }

    pub(crate) fn plus_current_health(&mut self) {
        if self.check_if_enough_score(self.current_health_increase_cost) {
            if self.current_health < self.max_health {
                self.current_health += 1;
                self.remove_score(self.current_health_increase_cost);
            }
        }
    }

    pub(crate) fn upgrade_scan_speed(&mut self) {
        if self.check_if_enough_score(self.scan_speed_upgrade_cost) {
            self.scan_speed.0 -= self.scan_speed.2;
            self.remove_score(self.scan_speed_upgrade_cost);
        }
    }

    pub(crate) fn upgrade_shield_time(&mut self) {
        if self.check_if_enough_score(self.shield_time_upgrade_cost) {
            self.shield_cost_rate += 1.;
            self.remove_score(self.shield_time_upgrade_cost);
        }
    }

    //score related stuff
    pub(crate) fn add_score(&mut self, amount: u32) {
        self.score += amount;
    }

    pub(crate) fn lock_remaining_score(&mut self) {
        self.locked_score += self.score;
        self.score = 0;
    }

    pub(crate) fn check_if_enough_score(&mut self, cost: u32) -> bool {
        if self.score >= cost {
            return true;
        }
        return false;
    }

    pub(crate) fn remove_score(&mut self, amount: u32) {
        if self.score as i32 - amount as i32 <= 0 {
            self.score = 0;
        } else {
            self.score -= amount;
        }
    }

    //health stuff
    pub(crate) fn damage(&mut self) -> bool {
        self.current_health -= 1;
        if self.current_health <= 0 {
            return true;
        }
        return false;
    }

    pub(crate) fn add_health(&mut self) {
        self.current_health += 1;
        if self.current_health >= self.max_health {
            self.current_health = self.max_health
        }
    }
}

fn setup_player(mut commands: Commands, sprites: Res<AssetHolder>) {
    commands.spawn_bundle(PlayerBundle::new(sprites));
}

pub struct ScoreEvent(pub(crate) u32);

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
        if player_stats.time_till_next_energy >= player_stats.energy_recharge_rate.0 {
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
                commands.insert_resource(NextState(GameState::Lose));
            }
        }
    }
}

fn handle_restart_game_events(mut commands: Commands) {
    commands.insert_resource(PlayerStats::default());
}

fn handle_score_events(
    mut score_event: EventReader<ScoreEvent>,
    mut player_stats: ResMut<PlayerStats>,
    mut commands: Commands,
) {
    for event in score_event.iter() {
        player_stats.add_score(event.0);
    }
}
