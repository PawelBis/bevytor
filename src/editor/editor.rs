use crate::editor::asset_loader::{AssetLoaderPlugin, GameAssets};
use crate::WindowSize;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_egui::egui::style::DebugOptions;
use bevy_egui::egui::style::Margin;
use bevy_egui::{egui, EguiContext};

struct Initialized {
    is: bool,
}

pub struct EditorPlugins;
impl PluginGroup for EditorPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(AssetLoaderPlugin).add(AssetWindowPlugin);
    }
}

pub struct AssetWindowPlugin;
impl Plugin for AssetWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(content_browser_system);
    }
}

const CONTENT_BROWSER_THUMBNAIL_SIZE: egui::Vec2 = egui::Vec2::new(150.0, 150.0);
fn content_browser_system(mut egui_context: ResMut<EguiContext>, game_assets: Res<GameAssets>) {
    let draw_image_line = |ui: &mut egui::Ui| {
        ui.with_layout(
            egui::Layout::left_to_right()
                .with_cross_align(egui::Align::Min)
                .with_main_wrap(true),
            |ui| {
                // TODO: Use space left for sidebar
                let avail_space = ui.available_size_before_wrap();
                let images_per_row_raw = avail_space.x / CONTENT_BROWSER_THUMBNAIL_SIZE.x;
                let space_left = avail_space.x * images_per_row_raw.fract();

                for img in game_assets.images.values() {
                    let image_button = egui::widgets::ImageButton::new(
                        img.egui_texture_id,
                        CONTENT_BROWSER_THUMBNAIL_SIZE,
                    );
                    ui.add(image_button);
                }
            },
        );
        true
    };

    egui::panel::TopBottomPanel::bottom("ConentBrowserPanel")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_image_line(ui);
                });
        });
}
