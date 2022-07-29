use crate::editor::asset_loader::{load_assets_system, AssetLoaderPlugin, GameAssets};
use crate::editor::editor::EditorPlugins;
use crate::mouse_input::MouseInputPlugin;
use crate::units::UnitsPlugin;
use bevy::prelude::*;
use bevy::window::WindowResized;
use bevy_egui::EguiPlugin;

mod editor;
mod mouse_input;
mod units;

#[derive(Component)]
struct MainCamera;

#[derive(Debug)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl WindowSize {
    fn to_vec(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "RTS Roguelike".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugins(EditorPlugins)
        .add_plugin(UnitsPlugin)
        .add_plugin(MouseInputPlugin)
        .add_startup_system(setup_system)
        //.add_system(editor_system)
        .add_system(window_resize_system)
        .run();
}

fn setup_system(mut commands: Commands, windows: Res<Windows>) {
    let window = windows
        .get_primary()
        .expect("Primary window should be valid!");
    let window_size = WindowSize {
        width: window.width(),
        height: window.height(),
    };
    commands.insert_resource(window_size);

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}

fn window_resize_system(
    mut windows_resize_events: EventReader<WindowResized>,
    mut window_size: ResMut<WindowSize>,
) {
    for event in windows_resize_events.iter() {
        window_size.width = event.width;
        window_size.height = event.height;
    }
}
