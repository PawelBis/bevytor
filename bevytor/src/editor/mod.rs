use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::prelude::SystemLabel;
use bevy_egui::EguiPlugin;
use assets::asset_loader::AssetLoaderPlugin;
use ui::asset_browser::AssetBrowserPlugin;
use crate::editor::commands::EditorCommandsPlugin;

pub mod assets;
pub mod ui;
pub mod commands;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum EditorStateLabel {
    InitializingAssets,
    PostInitializingAssets,
}

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