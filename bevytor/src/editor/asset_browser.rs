use bevy::app::{App, Plugin};
use bevy::asset::Asset;
use bevy::ecs::system::{Res, ResMut};
use bevy_egui::{
    egui::{Vec2, Layout, ColorImage, ImageButton, Ui, Align, panel::{TopBottomPanel, SidePanel}},
    EguiContext
};
use bevy_egui::egui::{CollapsingHeader, ScrollArea, TextureId};
use bevy_egui::egui::collapsing_header::CollapsingState;
use crate::asset_loader::{AssetDirectory, AssetType, EditorAssets};

pub struct AssetBrowserPlugin;
impl Plugin for AssetBrowserPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AssetBrowserSettings::default())
            .insert_resource(SelectedDirectory::default())
            .add_system(asset_browser_system);
    }
}

#[derive(Default, Debug)]
struct SelectedDirectory {
    details: Option<AssetDirectory>,
}

impl From<&AssetDirectory> for SelectedDirectory {
    fn from(other: &AssetDirectory) -> Self {
        let mut children_directories: Vec<AssetDirectory> = Vec::new();
        for child in other.children_directories.iter() {
            children_directories.push(AssetDirectory {
                name: child.name.clone(),
                path: child.path.clone(),
                children_directories: Vec::new(),
                assets: Vec::new(),
            });
        }

        SelectedDirectory {
            details: Some(AssetDirectory {
                name: other.name.clone(),
                path: other.path.clone(),
                children_directories,
                assets: Vec::from(other.assets.as_slice()),
            })
        }
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

fn draw_directory_hierarchy(
    mut ui: &mut Ui,
    asset_directory: &AssetDirectory,
    mut selected_directory: &mut SelectedDirectory,
) {
    let directory_name = &asset_directory.name.to_string_lossy().to_string();
    let id = ui.make_persistent_id(directory_name);
    CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            if ui.button(directory_name).clicked() {
                let new_selected: SelectedDirectory = asset_directory.into();
                selected_directory.details = new_selected.details;
            }
        })
        .body(|ui| {
            for child in asset_directory.children_directories.iter() {
                if child.children_directories.is_empty() {
                    if ui.button(child.name.to_string_lossy().to_string()).clicked() {
                        let new_selected: SelectedDirectory = asset_directory.into();
                        selected_directory.details = new_selected.details;
                    }
                } else {
                    draw_directory_hierarchy(ui, child, selected_directory);
                }
            }
        });
}

const DEFAULT_EGUI_MARGIN: Vec2 = Vec2::new(16.0, 16.0);
fn draw_assets(
    mut ui: &mut Ui,
    images_per_row: u32,
    asset_directory: &AssetDirectory,
    directory_texture: TextureId,
    mut selected_directory: &mut SelectedDirectory,
) {
    ui.with_layout(
        Layout::left_to_right()
            .with_cross_align(Align::Min)
            .with_main_wrap(true),
        |ui| {
            let available_space = ui.available_size_before_wrap();
            let thumbnail_size = available_space.x / images_per_row as f32;
                for d in asset_directory.children_directories.iter() {
                    //ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        let image_button = ImageButton::new(
                            directory_texture,
                            Vec2::new(thumbnail_size - DEFAULT_EGUI_MARGIN.x,
                                      thumbnail_size - DEFAULT_EGUI_MARGIN.y
                            )
                        );
                        if ui.add(image_button).double_clicked() {
                            selected_directory.details = SelectedDirectory::from(d).details;
                        };
                        //ui.label(d.name.to_string_lossy().to_string());
                    //});
                }

            for asset in asset_directory.assets.iter() {
                if let AssetType::Image(img) = asset {
                    //ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        let image_button = ImageButton::new(
                            img.egui_texture_id,
                            Vec2::new(thumbnail_size - DEFAULT_EGUI_MARGIN.x,
                                      thumbnail_size - DEFAULT_EGUI_MARGIN.y
                            )
                        );

                        ui.add(image_button);
                    //    ui.label(img.name.to_string_lossy().to_string());
                    //});
                }
            }
        },
    );
}

fn asset_browser_system(
    mut egui_context: ResMut<EguiContext>,
    mut settings: ResMut<AssetBrowserSettings>,
    mut assets_directory: ResMut<AssetDirectory>,
    mut selected_directory: ResMut<SelectedDirectory>,
    editor_assets: Res<EditorAssets>,
) {
    let mut ctx = egui_context.ctx_mut();
    let current_style = (*ctx.style()).clone();
    let mut new_style = current_style.clone();
    new_style.visuals.button_frame = false;
    ctx.set_style(new_style);
    TopBottomPanel::bottom("ContentBrowserPanel")
        .resizable(true)
        .show(ctx, |ui| {
            SidePanel::left("ContentBrowserFileSystem")
                .min_width(500.0)
                .show_inside(
                    ui,
                    |ui| ScrollArea::vertical().auto_shrink([false, false]).show(
                        ui, |ui| {
                            draw_directory_hierarchy(ui, &assets_directory, &mut selected_directory);
                        }
                    )
                );

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let selected = match &selected_directory.details {
                        Some(dir) => dir.clone(),
                        None => assets_directory.clone(),
                    };
                    draw_assets(ui, settings.thumbnails_per_row, &selected, editor_assets.directory_icon, &mut selected_directory);
                })
        });
    ctx.set_style(current_style);
}