use std::io::Cursor;
use bevy::prelude::*;
use crate::{MainCamera, WindowSize};

pub struct MouseInputPlugin;

#[derive(Default, Debug)]
pub struct CursorPosition {
    /// Position of the cursor in the world
    world: Vec2,
    /// Position of the cursor relative to the bottom-left corner of the window
    window: Vec2,
}

pub struct MoveCommandEvent {
    pub destination: Vec2,
}

impl Plugin for MouseInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MoveCommandEvent>()
            .insert_resource(CursorPosition::default())
            .add_system(mouse_input_system)
            .add_system(cursor_position_system);
    }
}

fn cursor_position_system(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window_size: Res<WindowSize>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = windows.get_primary().expect("Primary window should be valid!");

    if let Some(screen_position) = window.cursor_position() {
        let wsize = Vec2::new(window.width(), window.height());
        let ndc = (screen_position / wsize) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        cursor_position.world = world_pos.truncate();
        cursor_position.window = screen_position;
    }
}

fn mouse_input_system(
    mouse: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    mut move_event_writer: EventWriter<MoveCommandEvent>,
) {
    if mouse.pressed(MouseButton::Right) {
        move_event_writer.send(MoveCommandEvent {
            destination: cursor_position.world,
        });
    }
}