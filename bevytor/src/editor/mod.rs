use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::prelude::SystemLabel;
use bevy_egui::EguiPlugin;
use assets::asset_loader::AssetLoaderPlugin;
use ui::asset_browser::AssetBrowserPlugin;
use crate::editor::commands::EditorCommandsPlugin;

pub mod assets;
pub mod ui;
pub mod commands;

/// Label used for labeling editor dependent systems
/// EditorStateLabel::InitializingAssets - At this stage both editor and game assets are being
///                                         initialized and are not available
/// EditorStateLabel::PostInitializingAssets - At this stage both editor and game assets are available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum EditorStateLabel {
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
            .add(AssetBrowserPlugin)
            .add(EditorCommandsPlugin);
    }
}