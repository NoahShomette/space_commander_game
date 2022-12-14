use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

pub(crate) fn mouse_virtual_play_field_check(
    windows: &Res<Windows>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> bool {
    let (camera, _camera_transform) = camera_query.single();
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };
    return if let Some(screen_pos) = wnd.cursor_position() {
        let virtual_gameplay_size = Vec2::new(wnd.height() as f32, wnd.height() as f32);
        if screen_pos.x < (wnd.width() as f32 / 2. - virtual_gameplay_size.x / 2.)
            || screen_pos.x > (wnd.width() as f32 / 2. + virtual_gameplay_size.x / 2.)
        {
            false
        } else {
            true
        }
    } else {
        false
    };
}

pub(crate) fn mouse_screen_pos_to_world_pos(
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) -> Vec2 {
    let (camera, camera_transform) = camera_query.single();
    let mut mouse_pos: Vec2 = Default::default();

    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        mouse_pos = world_pos.truncate();
    }

    mouse_pos
}
