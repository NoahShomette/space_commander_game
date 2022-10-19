use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

use crate::enemy::enemy_difficulty::EnemyStats;
use crate::enemy::Enemy;
use crate::{AssetHolder, GameState};

pub(crate) struct EnemySpawnerPlugin;

impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        let mut fixed_update = SystemStage::parallel();

        fixed_update.add_system(
            spawn_next_wave
                // only do it in-game
                .run_in_state(GameState::Playing),
        );

        app.add_stage_before(
            CoreStage::Update,
            "FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_secs_f32(5.), fixed_update),
        );
    }
}

pub(crate) fn spawn_next_wave(
    sprites: Res<AssetHolder>,
    enemy_stats: Res<EnemyStats>,
    mut commands: Commands,
) {
    for i in 0..enemy_stats.amount_to_spawn_a_wave {
        Enemy::spawn(&sprites, &enemy_stats, &mut commands, &Vec2 { x: -700., y: 700. })
    }
}
