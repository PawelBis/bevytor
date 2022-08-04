use bevy::app::{App, Plugin};
use bevy::ecs::system::{Res, ResMut};
use bevy_egui::{
    egui::{Vec2, Layout, Ui, Align, panel::{TopBottomPanel, SidePanel}},
    EguiContext
};
use bevy_egui::egui::{ScrollArea, TextureId};
use bevy_egui::egui::collapsing_header::CollapsingState;
use crate::editor::assets::asset_loader::{
    AssetDirectory, AssetType, EditorAssets,
};
use crate::editor::ui::widgets;
use std::path::PathBuf;
use bevy::prelude::{Commands, EventReader, EventWriter, ParallelSystemDescriptorCoercion, };
use crate::editor::commands::{Command, CommandAny, CommandExecutedEvent, CommandExecuteMode, UndoRedoCommandEvent};
use crate::editor::EditorStateLabel;

#[derive(Debug)]
pub struct SelectDirectoryCommand {
    pub previous_selected_directory: AssetDirectory,
    pub new_selected_directory: AssetDirectory,
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
            .add_startup_system(
                selection_setup
                    .after(EditorStateLabel::InitializingAssets)
                    .label(EditorStateLabel::PostInitializingAssets)
            )
            .add_startup_system(selection_setup)
            .add_system(asset_browser_system)
            .add_system(select_directory_system);
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SelectedDirectory {
    details: AssetDirectory,
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
            details: AssetDirectory {
                name: other.name.clone(),
                path: other.path.clone(),
                children_directories,
                assets: Vec::from(other.assets.as_slice()),
            }
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

fn selection_setup(
    mut commands: Commands,
    root_directory: Res<AssetDirectory>,
) {
    let selected_directory: SelectedDirectory = root_directory.as_ref().into();
    commands.insert_resource(selected_directory);
}

// Remove recurrence!
fn draw_directory_hierarchy(
    ui: &mut Ui,
    asset_directory: &AssetDirectory,
) -> Option<PathBuf> {
    let directory_name = asset_directory.get_name();
    let id = ui.make_persistent_id(&directory_name);
    let mut new_selection: Option<PathBuf> = None;
    CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            let response = ui.button(directory_name);
            if response.clicked() {
                new_selection = Some(asset_directory.path.clone());
            }
        })
        .body(|ui| {
            for child in asset_directory.children_directories.iter() {
                if child.children_directories.is_empty() {
                    if ui.button(child.get_name()).clicked() {
                        new_selection = Some(child.path.clone());
                    }
                } else {
                    new_selection = draw_directory_hierarchy(ui, child);
                }
            }
        });

    new_selection
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
    currently_selected_directory: Res<SelectedDirectory>,
    editor_assets: Res<EditorAssets>,
    mut select_directory_event_writer: EventWriter<SelectDirectoryCommand>,
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
                            let new_selected_path = draw_directory_hierarchy(
                                ui,
                                &assets_directory);

                            if let Some(selected_path) = new_selected_path {
                                let new_selected_directory = assets_directory
                                    .find_by_path(&selected_path)
                                    .expect("Selected path is valid!");
                                let selected_command = SelectDirectoryCommand{
                                    previous_selected_directory: currently_selected_directory.details.clone(),
                                    new_selected_directory: new_selected_directory.clone(),
                                };
                                select_directory_event_writer.send(selected_command);
                            }
                        }
                    )
                );

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some (selected_path) = draw_assets(
                        ui,
                        settings.thumbnails_per_row,
                        &currently_selected_directory.details,
                        editor_assets.directory_icon,
                    ) {
                        if let Some(selected_dir) = assets_directory.find_by_path(&selected_path) {
                            let select_command = SelectDirectoryCommand {
                                new_selected_directory: selected_dir.clone(),
                                previous_selected_directory: currently_selected_directory.details.clone(),
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
        let new_dir = &event.new_selected_directory;
        if selected_directory.details != *new_dir {
            selected_directory.details = new_dir.clone();
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
            selected_directory.details = new_dir.clone();
        }
    }
}