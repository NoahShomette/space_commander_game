pub(crate) mod input;
mod player_missiles;
mod scanner;
mod shield;

use crate::player::input::input_manager::*;
use crate::player::player_missiles::player_missile_core::*;
use crate::player::scanner::scanner_core::*;
use crate::{AssetHolder, GameState, RestartGameEvent};

use crate::enemy::{Destroyed, Enemy};
use crate::player::shield::shield_core::ShieldPlugin;
use crate::sound::SoundEffectEvents;

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
                    .with_system(handle_player_energy_and_health_recharge)
                    .with_system(handle_time_score)
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
    pub(crate) is_auto_scan: bool,
    pub(crate) auto_scan_info: (f32, f32, f32, f32), // (current time till next scan, the target time till next scan, the min time you can set it to, the max time you can set it to)

    pub(crate) current_health: u32,
    pub(crate) max_health: u32,
    pub(crate) health_recharge_time: (f32, f32, f32),
    pub(crate) time_till_next_health: f32,

    pub(crate) is_regaining_energy: bool,
    pub(crate) max_energy: u32,
    pub(crate) current_energy: u32,
    pub(crate) energy_recharge_rate: (f32, f32, f32),
    pub(crate) time_till_next_energy: f32,
    pub(crate) energy_per_recharge: u32,

    pub(crate) missile_speed: (f32, f32, f32),
    pub(crate) missile_energy_cost: u32,

    pub(crate) current_points: u32,
    pub(crate) locked_score: u32,
    pub(crate) time_till_next_score: f32,

    pub(crate) scan_speed: (f32, f32, f32),
    pub(crate) scan_energy_cost: u32,

    pub(crate) shield_energy_cost: u32,
    pub(crate) shield_cost_rate: f32,

    pub(crate) enemy_kill_score: u32,

    //costs for upgrades
    pub(crate) max_energy_upgrade_cost: u32,
    pub(crate) energy_recharge_amount_upgrade_cost: u32,
    pub(crate) energy_recharge_rate_upgrade_cost: u32,

    pub(crate) missile_speed_upgrade_cost: u32,

    pub(crate) max_health_upgrade_cost: u32,
    pub(crate) current_health_increase_cost: u32,

    pub(crate) scan_speed_upgrade_cost: u32,

    pub(crate) shield_time_upgrade_cost: u32,

    pub(crate) is_cluster_missile_upgrade: bool,
    pub(crate) cluster_missile_upgrade_cost: u32,

    pub(crate) is_energy_vampire_upgrade: bool,
    pub(crate) energy_vampire_upgrade_cost: u32,

    pub(crate) is_dying_scanners_upgrade: bool,
    pub(crate) dying_scanners_upgrade_cost: u32,

    pub(crate) is_larger_missiles_upgrade: bool,
    pub(crate) larger_missiles_upgrade_cost: u32,

    pub(crate) all_time_score_count: u32,

    pub(crate) tutorial_panel: u32,
    pub(crate) max_tut_panel: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats {
            is_auto_scan: false,
            auto_scan_info: (0., 5.0, 1.0, 10.0),

            current_health: 2,
            max_health: 2,
            health_recharge_time: (30., 1., 1.),
            time_till_next_health: 0.,

            is_regaining_energy: true,
            max_energy: 6,
            current_energy: 6,
            energy_recharge_rate: (4.0, 0.4, 0.2),
            time_till_next_energy: 0.,
            energy_per_recharge: 1,

            missile_speed: (100., 500., 25.),
            missile_energy_cost: 1,

            current_points: 40000,
            locked_score: 0,
            time_till_next_score: 0.,

            scan_speed: (50.0, 200., 25.),
            scan_energy_cost: 2,

            shield_energy_cost: 1,
            shield_cost_rate: 1.0,
            enemy_kill_score: 5,

            //costs for upgrades
            max_energy_upgrade_cost: 15,
            energy_recharge_amount_upgrade_cost: 40,
            energy_recharge_rate_upgrade_cost: 20,

            missile_speed_upgrade_cost: 10,

            max_health_upgrade_cost: 20,
            current_health_increase_cost: 15,

            scan_speed_upgrade_cost: 10,

            shield_time_upgrade_cost: 10,

            is_cluster_missile_upgrade: false,
            cluster_missile_upgrade_cost: 200,

            is_energy_vampire_upgrade: false,
            energy_vampire_upgrade_cost: 200,

            is_dying_scanners_upgrade: false,
            dying_scanners_upgrade_cost: 200,

            is_larger_missiles_upgrade: false,
            larger_missiles_upgrade_cost: 200,

            all_time_score_count: 0,

            tutorial_panel: 0,
            max_tut_panel: 6,
        }
    }
}

impl PlayerStats {
    pub(crate) fn toggle_auto_scan(&mut self) {
        self.is_auto_scan = !self.is_auto_scan;
        self.auto_scan_info.0 = 0.;
    }

    pub(crate) fn auto_scan_tick(
        &mut self,
        time: Res<Time>,
        mut input_event_writer: EventWriter<PlayerInputEvents>,
    ) {
        if self.is_auto_scan {
            self.auto_scan_info.0 += time.delta_seconds();
            if self.auto_scan_info.0 >= self.auto_scan_info.1 {
                self.auto_scan_info.0 = 0.;
                input_event_writer.send(PlayerInputEvents::Scan);
            }
        }
    }

    pub(crate) fn recharge_energy(&mut self) -> bool {
        self.current_energy += self.energy_per_recharge;
        if self.current_energy > self.max_energy {
            self.current_energy = self.max_energy;
            return true;
        }
        return false;
    }

    pub(crate) fn plus_one_energy(&mut self) -> bool {
        self.current_energy += 1;
        if self.current_energy > self.max_energy {
            self.current_energy = self.max_energy;
            return true;
        }
        return false;
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
        self.auto_scan_info.0 = 0.;
    }

    pub(crate) fn shield_cost(&mut self) {
        self.current_energy -= self.shield_energy_cost;
    }

    //upgrades

    //ENERGY
    pub(crate) fn upgrade_max_energy(&mut self) -> bool {
        if self.check_if_enough_score(self.max_energy_upgrade_cost) {
            self.max_energy += 1;
            self.increase_all_time_score_count(self.max_energy_upgrade_cost);
            self.remove_score(self.max_energy_upgrade_cost);
            return true;
        }
        return false;
    }
    pub(crate) fn upgrade_energy_charge(&mut self) -> bool {
        if self.check_if_enough_score(self.energy_recharge_amount_upgrade_cost) {
            self.energy_per_recharge += 1;
            self.increase_all_time_score_count(self.energy_recharge_amount_upgrade_cost);
            self.remove_score(self.energy_recharge_amount_upgrade_cost);
            return true;
        }
        return false;
    }

    pub(crate) fn upgrade_energy_charge_speed(&mut self) -> bool {
        if self.check_if_enough_score(self.energy_recharge_rate_upgrade_cost)
            && self.energy_recharge_rate.0 > self.energy_recharge_rate.1
        {
            self.energy_recharge_rate.0 -= self.energy_recharge_rate.2;
            self.increase_all_time_score_count(self.energy_recharge_rate_upgrade_cost);
            self.remove_score(self.energy_recharge_rate_upgrade_cost);
            return true;
        }
        return false;
    }

    pub(crate) fn check_energy_recharge_speed_maxed(&mut self) -> bool {
        if self.energy_recharge_rate.0 <= self.energy_recharge_rate.1 {
            return true;
        }
        return false;
    }

    //HEALTH
    pub(crate) fn upgrade_max_health(&mut self) -> bool {
        if self.check_if_enough_score(self.max_health_upgrade_cost) {
            self.max_health += 1;
            self.heal();
            self.increase_all_time_score_count(self.max_health_upgrade_cost);
            self.remove_score(self.max_health_upgrade_cost);
            return true;
        }
        return false;
    }

    pub(crate) fn plus_current_health(&mut self) -> bool {
        if self.check_if_enough_score(self.current_health_increase_cost)
            && self.current_health < self.max_health
        {
            self.current_health += 1;
            self.increase_all_time_score_count(self.current_health_increase_cost);
            self.remove_score(self.current_health_increase_cost);
            return true;
        }
        return false;
    }

    pub(crate) fn heal(&mut self) -> bool {
        if self.current_health < self.max_health {
            self.current_health += 1;
            return true;
        }
        return false;
    }

    pub(crate) fn check_energy_full_health(&mut self) -> bool {
        if self.current_health < self.max_health {
            return false;
        }
        return true;
    }

    //SCAN
    pub(crate) fn upgrade_scan_speed(&mut self) -> bool {
        if self.check_if_enough_score(self.scan_speed_upgrade_cost)
            && self.scan_speed.0 <= self.scan_speed.1
        {
            self.scan_speed.0 += self.scan_speed.2;
            self.increase_all_time_score_count(self.scan_speed_upgrade_cost);
            self.remove_score(self.scan_speed_upgrade_cost);
            return true;
        }
        return false;
    }
    pub(crate) fn check_scan_speed_maxed(&mut self) -> bool {
        if self.scan_speed.0 >= self.scan_speed.1 {
            return true;
        }
        return false;
    }

    //SHIELD
    pub(crate) fn upgrade_shield_time(&mut self) -> bool {
        if self.check_if_enough_score(self.shield_time_upgrade_cost) {
            self.shield_cost_rate += 1.;
            self.increase_all_time_score_count(self.shield_time_upgrade_cost);
            self.remove_score(self.shield_time_upgrade_cost);
            return true;
        }
        return false;
    }

    //MISSILE
    pub(crate) fn upgrade_missile_speed(&mut self) -> bool {
        if self.check_if_enough_score(self.missile_speed_upgrade_cost)
            && self.missile_speed.0 < self.missile_speed.1
        {
            self.missile_speed.0 += self.missile_speed.2;
            self.increase_all_time_score_count(self.missile_speed_upgrade_cost);
            self.remove_score(self.missile_speed_upgrade_cost);
            return true;
        }
        return false;
    }

    pub(crate) fn check_missile_speed_maxed(&mut self) -> bool {
        if self.missile_speed.0 >= self.missile_speed.1 {
            return true;
        }
        return false;
    }

    //Super Upgrades
    //MISSILE
    pub(crate) fn upgrade_cluster_missile(&mut self) -> bool {
        if self.check_if_enough_score(self.cluster_missile_upgrade_cost)
            && self.is_cluster_missile_upgrade == false
        {
            self.is_cluster_missile_upgrade = true;
            self.increase_all_time_score_count(self.cluster_missile_upgrade_cost);
            self.remove_score(self.cluster_missile_upgrade_cost);
            return true;
        }
        return false;
    }
    pub(crate) fn upgrade_energy_vampire(&mut self) -> bool {
        if self.check_if_enough_score(self.energy_vampire_upgrade_cost)
            && self.is_energy_vampire_upgrade == false
        {
            self.is_energy_vampire_upgrade = true;
            self.increase_all_time_score_count(self.energy_vampire_upgrade_cost);
            self.remove_score(self.energy_vampire_upgrade_cost);
            return true;
        }
        return false;
    }
    pub(crate) fn upgrade_dying_scanners(&mut self) -> bool {
        if self.check_if_enough_score(self.dying_scanners_upgrade_cost)
            && self.is_dying_scanners_upgrade == false
        {
            self.is_dying_scanners_upgrade = true;
            self.increase_all_time_score_count(self.dying_scanners_upgrade_cost);
            self.remove_score(self.dying_scanners_upgrade_cost);
            return true;
        }
        return false;
    }
    pub(crate) fn upgrade_larger_missiles(&mut self) -> bool {
        if self.check_if_enough_score(self.larger_missiles_upgrade_cost)
            && self.is_larger_missiles_upgrade == false
        {
            self.is_larger_missiles_upgrade = true;
            self.increase_all_time_score_count(self.larger_missiles_upgrade_cost);
            self.remove_score(self.larger_missiles_upgrade_cost);
            return true;
        }
        return false;
    }

    //score related stuff
    pub(crate) fn add_score(&mut self, amount: u32) {
        self.current_points += amount;
        self.locked_score += amount;
    }

    pub(crate) fn lock_remaining_score(&mut self) -> bool {
        self.locked_score += self.current_points;
        self.increase_all_time_score_count(self.current_points);
        self.current_points = 0;
        return true;
    }

    pub(crate) fn check_if_enough_score(&mut self, cost: u32) -> bool {
        if self.current_points >= cost {
            return true;
        }
        return false;
    }

    pub(crate) fn remove_score(&mut self, amount: u32) {
        if self.current_points as i32 - amount as i32 <= 0 {
            self.current_points = 0;
        } else {
            self.current_points -= amount;
        }
    }

    //health stuff
    pub(crate) fn damage(&mut self) -> bool {
        if self.current_health as i32 - 1 as i32 <= 0 {
            self.current_health = 0;
            return true;
        } else {
            self.current_health -= 1;
        }
        return false;
    }

    pub(crate) fn increase_all_time_score_count(&mut self, amount: u32) {
        self.all_time_score_count += amount;
    }

    pub(crate) fn next_tut_panel(&mut self) {
        self.tutorial_panel += 1;
        if self.tutorial_panel >= self.max_tut_panel {
            self.tutorial_panel = self.max_tut_panel;
        }
    }

    pub(crate) fn prev_tut_panel(&mut self) {
        if self.tutorial_panel as i32 - 1 as i32 <= 0 {
            self.tutorial_panel = 0;
        } else {
            self.tutorial_panel -= 1;
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

pub fn handle_time_score(mut player_stats: ResMut<PlayerStats>, time: Res<Time>) {
    player_stats.time_till_next_score += time.delta_seconds();
    if player_stats.time_till_next_score >= 1. {
        player_stats.time_till_next_score -= 1.;
        player_stats.add_score(1);
    }
}

pub fn handle_player_energy_and_health_recharge(
    mut player_stats: ResMut<PlayerStats>,
    time: Res<Time>,
) {
    //health recharge
    if player_stats.current_health < player_stats.max_health {
        player_stats.time_till_next_health += time.delta_seconds();
        if player_stats.time_till_next_health >= player_stats.health_recharge_time.0 {
            player_stats.time_till_next_health = 0.;
            player_stats.heal();
        }
    }

    //energy recharge
    if player_stats.current_energy < player_stats.max_energy && player_stats.is_regaining_energy {
        player_stats.time_till_next_energy += time.delta_seconds();
        if player_stats.time_till_next_energy >= player_stats.energy_recharge_rate.0 {
            player_stats.time_till_next_energy = 0.;
            player_stats.recharge_energy();
        }
    }
}

pub(crate) fn handle_player_planet_collisions(
    mut missiles: Query<(&CollidingEntities), With<Player>>,
    mut enemy_entities: Query<&Enemy>,
    mut player_stats: ResMut<PlayerStats>,
    mut commands: Commands,
    mut sound_effect_writer: EventWriter<SoundEffectEvents>,
) {
    for entities in missiles.iter_mut() {
        for collision in entities.iter() {
            if let Ok(_enemy) = enemy_entities.get(collision) {
                commands.entity(_enemy.scan_ghost).despawn();
                commands.entity(collision).despawn();
                sound_effect_writer.send(SoundEffectEvents::PlanetDamaged);
                if player_stats.damage() {
                    commands.insert_resource(NextState(GameState::Lose));
                }
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
