use std::path::PathBuf;
use bevy::asset::Asset;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use crate::editor::assets::asset_loader::{AssetDirectory, EditorAssets};
use crate::editor::{EditorStateLabel, run_if_post_initializing_assets, run_if_post_initializing_assets_dbg};

pub struct ScenePickerPlugin;
impl Plugin for ScenePickerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MainScene::default())
            .add_startup_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_post_initializing_assets_dbg)
                    .with_system(create_scene_system)
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_post_initializing_assets)
                    .with_system(scene_picker_system)
            );
    }
}

#[derive(Default)]
pub struct MainScene {
    path: Option<PathBuf>,
}

pub fn scene_picker_system(
    scenes: Res<Assets<Scene>>,
    mut main_scene: ResMut<MainScene>,
    egui_context: ResMut<EguiContext>,
) {

}

pub fn create_scene_system(
    mut commands: Commands,
    editor_assets: Res<EditorAssets>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(SpriteBundle {
        texture: editor_assets.map_icon_handle.clone(),
        ..default()
    });
}
