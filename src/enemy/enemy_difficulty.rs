use crate::enemy::enemy_spawner::EnemySpawnerPlugin;
use crate::enemy::Enemy;
use crate::{GameState, PlayerStats, RestartGameEvent};
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub(crate) struct EnemyDifficultyPlugin;

impl Plugin for EnemyDifficultyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyStats>();
        app.add_system_set(
            ConditionSet::new()
                .with_system(handle_restart_game_events.run_on_event::<RestartGameEvent>())
                .into(),
        );
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .with_system(update_enemy_count)
                .into(),
        );

        let mut enemy_difficulty_fixed_update = SystemStage::parallel();

        enemy_difficulty_fixed_update.add_system(
            update_enemy_difficulty
                // only do it in-game
                .run_in_state(GameState::Playing),
        );
        app.add_stage_before(
            CoreStage::Update,
            "EnemyDifficultyFixedUpdate",
            FixedTimestepStage::from_stage(
                Duration::from_secs_f32(1.0),
                enemy_difficulty_fixed_update,
            ),
        );
    }
}

pub(crate) struct EnemyStats {
    pub(crate) speed: f32,

    pub(crate) time_between_waves: f32,
    pub(crate) amount_to_spawn_a_wave: u32,
    pub(crate) max_amount_to_spawn_a_wave: u32,
    pub(crate) upgrade_wave: bool,
    pub(crate) difficulty_level: u32,
    pub(crate) last_player_score_upgrade: u32,

    //microwave stuff
    pub(crate) amount_to_spawn_in_microwave: u32,
    pub(crate) time_between_microwaves: f32,
    pub(crate) time_till_next_microwave: f32,

    //assorted stuff
    pub(crate) current_enemy_amount: u32,

    //all time stats
    pub(crate) all_time_enemy_count: u32,
}

impl Default for EnemyStats {
    fn default() -> Self {
        EnemyStats {
            speed: 30.0,

            time_between_waves: 15.0,
            amount_to_spawn_a_wave: 1,
            max_amount_to_spawn_a_wave: 150,
            difficulty_level: 1,
            upgrade_wave: false,
            last_player_score_upgrade: 0,

            amount_to_spawn_in_microwave: 1,
            time_between_microwaves: 15.0,
            time_till_next_microwave: 0.0,

            current_enemy_amount: 0,
            all_time_enemy_count: 0,
        }
    }
}

fn handle_restart_game_events(mut commands: Commands) {
    commands.insert_resource(EnemyStats::default());
}

pub(crate) fn update_enemy_count(
    mut enemies: Query<Entity, With<Enemy>>,
    mut enemy_stats: ResMut<EnemyStats>,
) {
    let mut enemy_count = 0;
    for enemy in enemies.iter_mut() {
        enemy_count += 1;
    }
    enemy_stats.current_enemy_amount = enemy_count;
}

fn update_enemy_difficulty(
    mut enemy_stats: ResMut<EnemyStats>,
    player_stats: Res<PlayerStats>,
    time: Res<Time>,
    mut timesteps: ResMut<FixedTimestepInfo>,
) {
    timesteps.step = Duration::from_secs_f32(enemy_stats.time_between_waves);


    if enemy_stats.upgrade_wave {
        if (player_stats.locked_score - enemy_stats.last_player_score_upgrade) >= 500 {
            enemy_stats.amount_to_spawn_a_wave = enemy_stats.amount_to_spawn_a_wave * 2;
            enemy_stats.time_between_waves -= 2.0;
            enemy_stats.last_player_score_upgrade = player_stats.locked_score;
        } else {
            enemy_stats.amount_to_spawn_a_wave += 1;
            enemy_stats.time_between_waves -= 0.1;
        }

        enemy_stats.speed += 0.5;

        if enemy_stats.time_between_waves <= 3. {
            enemy_stats.time_between_waves = 3.;
        }
        enemy_stats.upgrade_wave = false;
        enemy_stats.difficulty_level += 1;
    } else {
        enemy_stats.upgrade_wave = true;
    }

    enemy_stats.time_between_microwaves = enemy_stats.time_between_waves / enemy_stats.amount_to_spawn_a_wave as f32;
    enemy_stats.amount_to_spawn_in_microwave = (enemy_stats.amount_to_spawn_a_wave as f32 / enemy_stats.time_between_microwaves).ceil() as u32;
    info!("{}{}",enemy_stats.time_between_microwaves, enemy_stats.amount_to_spawn_in_microwave );
}
