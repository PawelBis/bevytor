use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy_egui::EguiPlugin;
use assets::asset_loader::AssetLoaderPlugin;
use ui::asset_browser::AssetBrowserPlugin;
use crate::editor::commands::EditorCommandsPlugin;

pub mod assets;
pub mod ui;
pub mod commands;

pub struct EditorPlugins;
impl PluginGroup for EditorPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        let asset_browser_plugin = AssetBrowserPlugin;
        group
            .add(EguiPlugin)
            .add(AssetLoaderPlugin)
            .add_after::<AssetLoaderPlugin, AssetBrowserPlugin>(asset_browser_plugin)
            .add(EditorCommandsPlugin);
    }
}