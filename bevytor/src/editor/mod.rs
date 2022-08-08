use crate::editor::commands::EditorCommandsPlugin;
use assets::asset_loader::AssetLoaderPlugin;
use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::{ResMut, SystemLabel};
use bevy::scene::Scene;
use bevy_egui::EguiPlugin;
use ui::asset_browser::AssetBrowserPlugin;
use crate::editor::scene::scene_picker::ScenePickerPlugin;

pub mod assets;
pub mod commands;
pub mod scene;
pub mod ui;

fn run_if_post_initializing_assets(
    mut editor_state: ResMut<EditorStateLabel>
) -> ShouldRun {
    if *editor_state == EditorStateLabel::PostInitializingAssets {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

/// Label used for labeling editor dependent systems
/// EditorStateLabel::InitializingAssets - At this stage both editor and game assets are being
///                                         initialized and are not available
/// EditorStateLabel::PostInitializingAssets - At this stage both editor and game assets are available
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum EditorStateLabel {
    InitializingAssets,
    PostInitializingAssets,
}

/// This plugin group contains all the editor plugins and its dependencies, resulting in "complete" editor ui
pub struct EditorPlugins;
impl PluginGroup for EditorPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(EguiPlugin)
            .add(AssetLoaderPlugin)
            .add(EditorCommandsPlugin)
            .add(AssetBrowserPlugin)
            .add(ScenePickerPlugin);
    }
}
