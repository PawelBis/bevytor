use bevy::app::{App, Plugin};
use bevy::ecs::system::{Res, ResMut};
use bevy_egui::{
    egui::{Vec2, Layout, Ui, Align, panel::{TopBottomPanel, SidePanel}},
    EguiContext
};
use bevy_egui::egui::{ScrollArea, TextureId};
use bevy_egui::egui::collapsing_header::CollapsingState;
use crate::editor::assets::asset_loader::{AssetDirectory, AssetType, EditorAssets};
use crate::editor::ui::widgets;
use std::path::PathBuf;

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
    ui: &mut Ui,
    asset_directory: &AssetDirectory,
    selected_directory: &mut SelectedDirectory,
) {
    let directory_name = &asset_directory.name.to_string_lossy().to_string();
    let id = ui.make_persistent_id(directory_name);
    CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            let response = ui.button(directory_name);
            if response.clicked() {
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
    ui: &mut Ui,
    images_per_row: u32,
    asset_directory: &AssetDirectory,
    directory_texture: TextureId,
) -> Option<PathBuf> {
    let mut selected_directory_path: Option<PathBuf> = None;
    ui.with_layout(
        Layout::left_to_right()
            .with_cross_align(Align::Min)
            .with_main_wrap(true),
        |ui| {
            let available_space = ui.available_size_before_wrap();
            let thumbnail_size = available_space.x / images_per_row as f32;
                for d in asset_directory.children_directories.iter() {
                    if widgets::thumbnail(
                        ui,
                        d.name.to_string_lossy().to_string(),
                        Vec2::splat(thumbnail_size) - DEFAULT_EGUI_MARGIN, directory_texture
                    ).double_clicked() {
                        selected_directory_path = Some(d.path.to_path_buf());
                    }
                }

            for asset in asset_directory.assets.iter() {
                let AssetType::Image(img) = asset;
                let thumbnail = widgets::Thumbnail {
                    label: img.name.to_string_lossy().to_string(),
                    size: Vec2::splat(thumbnail_size) - DEFAULT_EGUI_MARGIN,
                    texture_id: img.egui_texture_id,
                    selected: false,
                    ..Default::default()
                };
                ui.add(thumbnail);
            }
        },
    );

    selected_directory_path
}

fn asset_browser_system(
    mut egui_context: ResMut<EguiContext>,
    settings: ResMut<AssetBrowserSettings>,
    assets_directory: ResMut<AssetDirectory>,
    mut selected_directory: ResMut<SelectedDirectory>,
    editor_assets: Res<EditorAssets>,
) {
    let ctx = egui_context.ctx_mut();
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
                    if let Some (selected_path) = draw_assets(
                        ui,
                        settings.thumbnails_per_row,
                        &selected,
                        editor_assets.directory_icon,
                    ) {
                        println!("{:?}", selected_path);
                        if let Some(selected_dir) = assets_directory.find_by_path(&selected_path) {
                            selected_directory.details = SelectedDirectory::from(selected_dir).details;
                        }
                    };
                })
        });
    ctx.set_style(current_style);
}