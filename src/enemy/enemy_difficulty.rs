use crate::enemy::enemy_spawner::EnemySpawnerPlugin;
use crate::enemy::Enemy;
use crate::{GameState, PlayerStats};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub(crate) struct EnemyDifficultyPlugin;

impl Plugin for EnemyDifficultyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyStats>();
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .after("main_enemy_stat_loop")
                .with_system(update_enemy_count)
                .into(),
        );
    }
}

pub(crate) struct EnemyStats {
    pub(crate) speed: f32,
    pub(crate) amount_to_spawn_a_wave: u32,

    pub(crate) difficulty_leve: u32,

    //assorted stuff
    pub(crate) current_enemy_amount: u32,

    //all time stats
    pub(crate) all_time_enemy_count: u32,
}

impl Default for EnemyStats {
    fn default() -> Self {
        EnemyStats {
            speed: 50.0,
            amount_to_spawn_a_wave: 1,

            difficulty_leve: 1,
            
            current_enemy_amount: 0,
            all_time_enemy_count: 0,
        }
    }
}

fn handle_restart_game_events(
    mut commands: Commands,
) {
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
