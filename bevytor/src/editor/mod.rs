use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::ecs::system::Commands;
use bevy::asset::{AssetServer, Handle};
use bevy::render::texture::Image;
use bevy::ecs::system::ResMut;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_egui::egui::TextureId;
use std::env;
use crate::asset_loader::AssetLoaderPlugin;
use crate::editor::asset_browser::AssetBrowserPlugin;

pub mod asset_browser;

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