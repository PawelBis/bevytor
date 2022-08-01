use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy_egui::EguiPlugin;
use crate::asset_loader::AssetLoaderPlugin;
use crate::editor::asset_browser::AssetBrowserPlugin;

pub mod asset_browser;
pub mod widgets;

pub struct EditorPlugins;
impl PluginGroup for EditorPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        let asset_browser_plugin = AssetBrowserPlugin;
        group
            .add(EguiPlugin)
            .add(AssetLoaderPlugin)
            .add_after::<AssetLoaderPlugin, AssetBrowserPlugin>(asset_browser_plugin);
    }
}