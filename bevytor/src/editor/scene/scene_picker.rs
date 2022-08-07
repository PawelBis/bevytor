use std::path::PathBuf;
use bevy::asset::Asset;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use crate::editor::assets::asset_loader::AssetDirectory;
use crate::editor::EditorStateLabel;

pub struct ScenePickerPlugin;
impl Plugin for ScenePickerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MainScene::default())
            .add_system(scene_picker_system.after(EditorStateLabel::InitializingAssets));
    }
}

#[derive(Default)]
pub struct MainScene {
    path: Option<PathBuf>,
}

fn scene_picker_system(
    scenes: Res<Assets<Scene>>,
    mut main_scene: ResMut<MainScene>,
    egui_context: ResMut<EguiContext>,
) {

}