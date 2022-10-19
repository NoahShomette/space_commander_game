use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub(crate) struct EnemyStats {
    pub(crate) speed: f32,
    pub(crate) amount_to_spawn_a_wave: u32,
}

impl FromWorld for EnemyStats {
    fn from_world(_world: &mut World) -> Self {
        EnemyStats {
            speed: 50.0,
            amount_to_spawn_a_wave: 1,
        }
    }
}
