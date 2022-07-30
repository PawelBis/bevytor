use bevy::app::{App, Plugin};
use bevy::ecs::system::{Res, ResMut};
use bevy_egui::{
    egui::{Vec2, Layout, Ui, Align, panel::{TopBottomPanel, SidePanel}},
    EguiContext
};

pub struct AssetBrowserPlugin;
impl Plugin for AssetBrowserPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AssetBrowserSettings::default())
            .add_system(asset_browser_system);
    }
}

struct AssetBrowserSettings {
    thumbnails_per_row: u32,
}

impl Default for AssetBrowserSettings {
    fn default() -> Self {
        Self {
            thumbnails_per_row: 8,
        }
    }
}

const DEFAULT_EGUI_MARGIN: Vec2 = Vec2::new(16.0, 16.0);
fn draw_images(
    mut ui: &mut Ui,
    images_per_row: u32,
) {
    ui.with_layout(
        Layout::left_to_right()
            .with_cross_align(Align::Min)
            .with_main_wrap(true),
        |ui| {
            let available_space = ui.available_size_before_wrap();
            let thumbnail_size = available_space.x / 8.0;
            //for img in game_assets.images.values() {
            //    let image_button = egui::widgets::ImageButton::new(
            //        img.egui_texture_id,
            //        Vec2::new(thumbnail_size - DEFAULT_EGUI_MARGIN.x,
            //                  thumbnail_size - DEFAULT_EGUI_MARGIN.y
            //        )
            //    );

            //    ui.add(image_button);
            //}
        },
    );
}

fn asset_browser_system(
    mut egui_context: ResMut<EguiContext>,
    mut settings: ResMut<AssetBrowserSettings>,
) {

    TopBottomPanel::bottom("ConentBrowserPanel")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            SidePanel::left("ContentBrowserFileSystem")
                .min_width(500.0)
                .show_inside(ui, |ui| {});

            //egui::ScrollArea::vertical()
            //    .auto_shrink([false, false])
            //    .show(ui, |ui| draw_images(ui, &game_assets, settings.thumbnails_per_row));
        });
}