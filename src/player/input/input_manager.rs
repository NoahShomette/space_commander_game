use bevy::input::keyboard::KeyboardInput;
use crate::GameState;

use crate::helpers::{mouse_screen_pos_to_world_pos, mouse_virtual_play_field_check};
use crate::player::player_missiles::player_missile_core::*;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub(crate) struct PlayerInputPlugin;

//#[derive(Default)]
pub(crate) enum PlayerInputEvents {
    FireMissile(Vec2),
    Scan,
    Shield(bool),
}

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerInputEvents>()

            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .label("player_input")
                    .with_system(player_input)
                    .into(),
            );

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::MainMenu)
                .with_system(if_start_game)
                .into()
        );
    }
}

pub fn if_start_game(keyboard_input: Res<Input<KeyCode>>, mut commands: Commands) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.insert_resource(NextState(GameState::Playing));
    }
}

pub(crate) fn player_input(
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut input_event_writer: EventWriter<PlayerInputEvents>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if mouse_virtual_play_field_check(&windows, &camera_query) {
            let mouse_world_pos = mouse_screen_pos_to_world_pos(windows, camera_query);
            input_event_writer.send(PlayerInputEvents::FireMissile(mouse_world_pos));
        }
    }

    if mouse_input.just_pressed(MouseButton::Right) {
        input_event_writer.send(PlayerInputEvents::Scan);
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        input_event_writer.send(PlayerInputEvents::Shield(true));
    }
    if keyboard_input.just_released(KeyCode::Space) {
        input_event_writer.send(PlayerInputEvents::Shield(false));
    }
}
