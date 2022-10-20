use crate::enemy::{Destroyed, Enemy};
use crate::player::input::input_manager::PlayerInputEvents;
use crate::player::scanner::scanner_core::scan;
use crate::{AssetHolder, GameState, Player, PlayerStats};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::FillMode;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub(crate) struct ShieldPlugin;

#[derive(Component)]
pub(crate) struct ShieldComp;

#[derive(Component)]
pub(crate) struct ShieldRes {
    is_active: bool,
    time_till_next_cost: f32,
}

impl FromWorld for ShieldRes {
    fn from_world(_world: &mut World) -> Self {
        ShieldRes {
            is_active: false,
            time_till_next_cost: 0.0,
        }
    }
}

impl Plugin for ShieldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 4 })
            .add_plugin(ShapePlugin)
            .init_resource::<ShieldRes>();

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("shield_loop")
                .with_system(handle_player_shield_events.run_on_event::<PlayerInputEvents>())
                .with_system(shield_count_cost.run_if(is_shield_active))
                .with_system(handle_player_planet_collisions.run_if(is_shield_active))
                .into(),
        );

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("shield_loop")
                .into(),
        );
    }
}

fn is_shield_active(shield_resource: Res<ShieldRes>) -> bool {
    shield_resource.is_active
}

fn shield_count_cost(
    time: Res<Time>,
    mut player_stats: ResMut<PlayerStats>,
    mut shield_resource: ResMut<ShieldRes>,
    shield_query: Query<Entity, With<ShieldComp>>,
    mut commands: Commands,
) {
    shield_resource.time_till_next_cost += time.delta_seconds();
    if shield_resource.time_till_next_cost >= player_stats.shield_cost_rate {
        if player_stats.check_if_enough_energy(player_stats.shield_energy_cost) {
            shield_resource.time_till_next_cost -= player_stats.shield_cost_rate;
            player_stats.shield_cost();
            shield(&mut commands);
        } else {
            player_stats.is_regaining_energy = true;
            shield_resource.is_active = false;
            remove_shield(&shield_query, &mut commands);
        }
    }
}

pub(crate) fn handle_player_shield_events(
    mut shield_resource: ResMut<ShieldRes>,
    mut player_stats: ResMut<PlayerStats>,
    shield_query: Query<Entity, With<ShieldComp>>,
    mut commands: Commands,
    mut player_input_event_reader: EventReader<PlayerInputEvents>,
) {
    for event in player_input_event_reader.iter() {
        match event {
            PlayerInputEvents::FireMissile(_) => {}
            PlayerInputEvents::Scan => {}
            PlayerInputEvents::Shield(state) => {
                if *state == true {
                    if player_stats.check_if_enough_energy(player_stats.shield_energy_cost) {
                        player_stats.is_regaining_energy = false;
                        shield_resource.is_active = true;
                        shield_resource.time_till_next_cost = 0.0;
                        player_stats.shield_cost();
                        shield(&mut commands);
                    }
                } else {
                    player_stats.is_regaining_energy = true;
                    shield_resource.is_active = false;
                    remove_shield(&shield_query, &mut commands);
                }
            }
        }
    }
}

pub(crate) fn shield(mut commands: &mut Commands) {
    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Circle {
                radius: 30.0,
                center: Default::default(),
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                }),
                outline_mode: StrokeMode::new(
                    Color::Rgba {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                    3.0,
                ),
            },
            Transform::default(),
        ))
        .insert(ShieldComp)
        .insert(Collider::ball(30.0))
        .insert(Sensor);
}

pub(crate) fn remove_shield(
    shield_query: &Query<Entity, With<ShieldComp>>,
    mut commands: &mut Commands,
) {
    for entity in shield_query.iter() {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn handle_player_planet_collisions(
    mut shield: Query<(&CollidingEntities), With<ShieldComp>>,
    mut enemy_entities: Query<&Enemy>,
    mut commands: Commands,
) {
    let shield = shield.single_mut();
    for collision in shield.iter() {
        if let Ok(_enemy) = enemy_entities.get(collision) {
            commands.entity(collision).insert(Destroyed);
        }
    }
}
