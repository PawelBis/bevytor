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
use bevy::prelude::{EventReader, EventWriter};
use bevy::prelude::KeyCode::Comma;
use crate::editor::commands::{Command, CommandAny, CommandExecutedEvent, CommandExecuteMode, UndoRedoCommandEvent};

#[derive(Debug)]
pub struct SelectDirectoryCommand {
    pub previous_selected_directory: Option<AssetDirectory>,
    pub new_selected_directory: Option<AssetDirectory>,
}

impl Command for SelectDirectoryCommand {
    fn recreate(&self) -> Box<dyn CommandAny> {
       Box::new(Self {
           previous_selected_directory: self.previous_selected_directory.clone(),
           new_selected_directory: self.new_selected_directory.clone(),
       })
    }

    fn command_type(&self) -> &str {
        "select_directory_command"
    }
}

pub struct AssetBrowserPlugin;
impl Plugin for AssetBrowserPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SelectDirectoryCommand>()
            .insert_resource(AssetBrowserSettings::default())
            .insert_resource(SelectedDirectory::default())
            .add_system(asset_browser_system)
            .add_system(select_directory_system);
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
struct SelectedDirectory {
    details: Option<AssetDirectory>,
}
impl SelectedDirectory {
    fn is_valid(&self) -> bool {
        self.details.is_some()
    }
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

impl From<AssetDirectory> for SelectedDirectory {
    fn from(other: AssetDirectory) -> Self {
        Self::from(&other)
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

// Remove recurency!
fn draw_directory_hierarchy(
    ui: &mut Ui,
    asset_directory: &AssetDirectory,
    selected_directory: &mut SelectedDirectory,
    mut selected_command_writer: &mut EventWriter<SelectDirectoryCommand>,
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
                    draw_directory_hierarchy(ui, child, selected_directory, selected_command_writer);
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
    selected_directory: Res<SelectedDirectory>,
    editor_assets: Res<EditorAssets>,
    mut select_directory_event_writer: EventWriter<SelectDirectoryCommand>,
) {
    let ctx = egui_context.ctx_mut();
    let current_style = (*ctx.style()).clone();
    let mut new_style = current_style.clone();
    let old_selected: SelectedDirectory = match &selected_directory.details {
        Some(dir) => dir.into(),
        None => assets_directory.as_ref().into(),
    };
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
                            //println!("Old selection: {:?}", old_selected);
                            let mut new_selection = SelectedDirectory::default();
                            draw_directory_hierarchy(ui, &assets_directory, &mut new_selection, &mut select_directory_event_writer);
                            if new_selection.is_valid() {
                                let selected_command = SelectDirectoryCommand{
                                    previous_selected_directory: old_selected.clone().details,
                                    new_selected_directory: new_selection.details,
                                };
                                select_directory_event_writer.send(selected_command);
                            }
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
                        if let Some(selected_dir) = assets_directory.find_by_path(&selected_path) {
                            println!("Sele {:?}", selected_dir);
                            let select_command = SelectDirectoryCommand {
                                new_selected_directory: Some(selected_dir.clone()),
                                previous_selected_directory: old_selected.clone().details,
                            };
                            select_directory_event_writer.send(select_command);
                        }
                    };
                })
        });
    ctx.set_style(current_style);
}

fn select_directory_system(
    mut normal_reader: EventReader<SelectDirectoryCommand>,
    mut undo_redo_reader: EventReader<UndoRedoCommandEvent>,
    mut selected_directory: ResMut<SelectedDirectory>,
    mut command_executed_writer: EventWriter<CommandExecutedEvent>,
) {
    for event in normal_reader.iter() {
        println!("New event {:?}", event);
        let new_dir: SelectedDirectory = event
            .new_selected_directory
            .as_ref()
            .unwrap()
            .into();
        if *selected_directory != new_dir {
            selected_directory.details = new_dir.details;
            command_executed_writer.send(CommandExecutedEvent {
                inner: event.recreate(),
            });
        }
    }

    for undo_redo_event in undo_redo_reader.iter() {
        if undo_redo_event.command_type() == "select_directory_command" {
            let selected_directory_command: &SelectDirectoryCommand = undo_redo_event
                .inner
                .as_any()
                .downcast_ref()
                .unwrap();
            let new_dir = match undo_redo_event.mode {
                CommandExecuteMode::Redo => { &selected_directory_command.new_selected_directory }
                CommandExecuteMode::Undo => { &selected_directory_command.previous_selected_directory }
            };
            let new_selected_dir: SelectedDirectory = new_dir.as_ref().unwrap().into();
            selected_directory.details = new_selected_dir.details;
        }
    }
}