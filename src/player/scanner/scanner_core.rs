use crate::enemy::{Destroyed, Enemy, Scanned};
use crate::player::input::input_manager::PlayerInputEvents;
use crate::{AssetHolder, GameState, PlayerStats};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::FillMode;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub(crate) struct ScannerPlugin;

impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 4 })
            .add_plugin(ShapePlugin);
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .label("scan_loop")
                .before("main_enemy_loop")
                .with_system(handle_player_scan_spawn_events.run_on_event::<PlayerInputEvents>())
                .with_system(increase_scan_radius)
                .with_system(handle_scanner_collisions)
                .into(),
        );
        /*
            let mut fixedupdate = SystemStage::parallel();
            fixedupdate.add_system(
                increase_scan_radius
                    // only do it in-game
                    .run_in_state(GameState::Playing),
            );

            app.insert_resource(Msaa { samples: 4 })
                .add_plugin(ShapePlugin)
                .add_stage_before(
                    CoreStage::Update,
                    "FixedUpdate",
                    FixedTimestepStage::from_stage(Duration::from_millis(10), fixedupdate),
                )
                .add_enter_system(GameState::Playing, scan);
        }*/
    }
}

#[derive(Component)]
pub(crate) struct Scan {
    size: f32,
}

pub(crate) fn handle_player_scan_spawn_events(
    sprites: Res<AssetHolder>,
    mut player_stats: ResMut<PlayerStats>,
    mut commands: Commands,
    mut player_input_event_reader: EventReader<PlayerInputEvents>,
) {
    for event in player_input_event_reader.iter() {
        match event {
            PlayerInputEvents::FireMissile(_) => {}
            PlayerInputEvents::Scan => {
                if player_stats.check_if_enough_energy(player_stats.scan_energy_cost) {
                    player_stats.scanner_fired();
                    scan(&mut commands);
                }
            }
            PlayerInputEvents::Shield(_) => {}
        }
    }
}

pub(crate) fn scan(mut commands: &mut Commands) {
    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Circle {
                radius: 10.0,
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
                        red: 0.3,
                        green: 0.6,
                        blue: 0.25,
                        alpha: 1.0,
                    },
                    3.0,
                ),
            },
            Transform::default(),
        ))
        .insert(CollidingEntities::default())
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Scan { size: 20.0 })
        .insert(Collider::ball(20.0))
        .insert(Sensor);
}

pub(crate) fn increase_scan_radius(
    mut query: Query<(Entity, &mut Path, &mut Scan, &mut Collider)>,
    time: Res<Time>,
    player_stats: Res<PlayerStats>,
    mut commands: Commands,
) {
    for (entity, mut path, mut scan, mut collider) in query.iter_mut() {
        scan.size += player_stats.scan_speed * time.delta_seconds();
        let new_size = scan.size + player_stats.scan_speed * time.delta_seconds();
        let new_circle = shapes::Circle {
            radius: new_size,
            center: Default::default(),
        };

        if scan.size >= 1000. {
            commands.entity(entity).despawn();
        }

        *path = ShapePath::build_as(&new_circle);
        *collider = Collider::ball(scan.size);
    }
}

pub(crate) fn handle_scanner_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    scans: Query<(&CollidingEntities), With<Scan>>,
    scan: Query<&Scan>,
    enemy_entities: Query<&Enemy>,
    mut commands: Commands,
) {
    //using collision events so that we can have it only activate on start
    //we iterate through all the collision events - then we match that to only get the starting ones
    //afterwards we compare the two entities and see if one is a scan and one is an enemy. If so then
    //scan them
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                if let Ok(_enemy) = enemy_entities.get(*a) {
                    if let Ok(_scan) = scan.get(*b) {
                        info!("did scan an enemy");
                        commands.entity(*a).insert(Scanned);
                    }
                }
                if let Ok(_enemy) = enemy_entities.get(*b) {
                    if let Ok(_scan) = scan.get(*a) {
                        info!("did scan an enemy");
                        commands.entity(*b).insert(Scanned);
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => {}
        }
    }
    /*
    for entities in scans.iter() {
        for collision in entities.iter() {
            if let Ok(_enemy) = enemy_entities.get(collision) {
                info!("did scan an enemy");
                commands.entity(collision).insert(Scanned);
            }
        }
        */
}
