use crate::editor::assets::asset_loader::{AssetDirectory, AssetType, EditorAssets};
use crate::editor::commands::{
    Command, CommandAny, CommandExecuteDirection, ExecuteCommandEvent, UndoRedoCommandEvent,
};
use crate::editor::run_if_post_initializing_assets;
use crate::editor::ui::widgets::{self, draw_directory_hierarchy};
use bevy::app::{App, Plugin};
use bevy::ecs::system::{Res, ResMut};
use bevy::prelude::{Commands, EventReader, EventWriter, SystemSet};
use bevy_egui::egui::ScrollArea;
use bevy_egui::{
    egui::{
        panel::{SidePanel, TopBottomPanel},
        Align, Layout, Ui, Vec2,
    },
    EguiContext,
};
use std::any::TypeId;
use std::path::PathBuf;

#[derive(Clone)]
pub enum Selection {
    Directory(PathBuf),
    Asset(AssetType),
}

// TODO: Use this in asset_browser_system to propagate EnterDirectory and MainAssetCommand commands
pub enum SelectionCommand {
    Directory(EnterDirectoryCommand),
    Asset,
}

/// Command used for notification about SelectDirectory events.
/// Designed with support for Undo and Redo in mind
#[derive(Debug)]
pub struct EnterDirectoryCommand {
    pub previous_selected_directory: PathBuf,
    pub new_selected_directory: PathBuf,
}

impl Command for EnterDirectoryCommand {
    fn recreate(&self) -> Box<dyn CommandAny> {
        Box::new(Self {
            previous_selected_directory: self.previous_selected_directory.clone(),
            new_selected_directory: self.new_selected_directory.clone(),
        })
    }

    fn command_type(&self) -> TypeId {
        TypeId::of::<EnterDirectoryCommand>()
    }
}

/// Plugin for displaying and manipulating assets in file system like manner.
/// UnrealEngine content browser is main inspiration
pub struct AssetBrowserPlugin;
impl Plugin for AssetBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnterDirectoryCommand>()
            .insert_resource(AssetBrowserSettings::default())
            .insert_resource(SelectedDirectory::default())
            .add_startup_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_post_initializing_assets)
                    .with_system(selection_setup),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_post_initializing_assets)
                    .with_system(asset_browser_system)
                    .with_system(select_directory_system),
            );
    }
}

/// Resource containing shallow copy of currently selected directory
#[derive(Debug, Eq, PartialEq)]
pub struct SelectedDirectory {
    details: AssetDirectory,
}

impl Default for SelectedDirectory {
    fn default() -> Self {
        Self {
            details: AssetDirectory {
                name: "Dummy".into(),
                path: PathBuf::default(),
                children_directories: Vec::new(),
                assets: Vec::new(),
            },
        }
    }
}

impl Clone for SelectedDirectory {
    // Just abuse the fact that we can recreate itself from our inner AssetDirectory
    fn clone(&self) -> Self {
        Self::from(&self.details)
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
            details: AssetDirectory {
                name: other.name.clone(),
                path: other.path.clone(),
                children_directories,
                assets: Vec::from(other.assets.as_slice()),
            },
        }
    }
}

impl From<AssetDirectory> for SelectedDirectory {
    fn from(other: AssetDirectory) -> Self {
        Self::from(&other)
    }
}

impl SelectedDirectory {
    pub fn get_path(&self) -> PathBuf {
        self.details.get_path()
    }
}

/// Resource containing data about AssetBrowser settings
pub struct AssetBrowserSettings {
    /// Default height of the asset browser
    /// TODO: Change to use screen %
    default_height: f32,
    /// Number of thumbnails per row.
    /// AssetBrowser will scale all the thumbnails to satisfy this number
    thumbnails_per_row: u32,
    /// Default directory hierarchy width
    directory_hierarchy_widht: f32,
}

impl Default for AssetBrowserSettings {
    fn default() -> Self {
        Self {
            default_height: 200.0,
            thumbnails_per_row: 8,
            directory_hierarchy_widht: 350.0,
        }
    }
}

/// Setup system, right now only inserts SelectedDirectory resource. Should be moved to build function
pub fn selection_setup(
    mut _commands: Commands,
    root_directory: Res<AssetDirectory>,
    mut currently_selected_directory: ResMut<SelectedDirectory>,
    mut select_directory_event_writer: EventWriter<EnterDirectoryCommand>,
) {
    *currently_selected_directory = SelectedDirectory::from(root_directory.as_ref());
    let select_command = EnterDirectoryCommand {
        new_selected_directory: root_directory.get_path(),
        previous_selected_directory: currently_selected_directory.get_path(),
    };
    select_directory_event_writer.send(select_command);
}

/// As name suggests....
/// Draws all the directories and assets contained within currently
/// selected directory (Res<SelectedDirectory>)
const DEFAULT_EGUI_MARGIN: Vec2 = Vec2::new(16.0, 16.0);
fn draw_assets(
    ui: &mut Ui,
    images_per_row: u32,
    asset_directory: &AssetDirectory,
    editor_assets: &EditorAssets,
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
                    Vec2::splat(thumbnail_size) - DEFAULT_EGUI_MARGIN,
                    editor_assets.directory_icon,
                )
                .double_clicked()
                {
                    selected_directory_path = Some(d.path.to_path_buf());
                }
            }

            for asset in asset_directory.assets.iter() {
                let texture_id = match asset {
                    AssetType::Image(image) => image.egui_texture_id,
                    AssetType::Scene(_) => editor_assets.map_icon,
                };
                let thumbnail = widgets::Thumbnail {
                    label: asset.get_name(),
                    size: Vec2::splat(thumbnail_size) - DEFAULT_EGUI_MARGIN,
                    texture_id,
                    selected: false,
                    ..Default::default()
                };
                ui.add(thumbnail);
            }
        },
    );

    selected_directory_path
}

fn draw_side_panel_tree_view(
    ui: &mut Ui,
    root_directory: &AssetDirectory,
    width: f32,
) -> Option<Selection> {
    let mut new_selection: Option<Selection> = None;
    let draw_hierarchy = |ui: &mut Ui| {
        let potential_selection = draw_directory_hierarchy(ui, &root_directory, false);
        if let Some(selection) = potential_selection {
            new_selection = Some(selection);
        }
    };

    let side_panel = SidePanel::left("ContentBrowserTreeView").default_width(width);
    let scroll_area = ScrollArea::vertical().auto_shrink([false, false]);

    side_panel.show_inside(ui, |ui| scroll_area.show(ui, draw_hierarchy));

    new_selection
}

/// System drawing the asset browser. Contains mostly layout and commands.
/// Uses helper functions (draw_assets, draw_directory_hierarchy) and draw for detailed drawings
pub fn asset_browser_system(
    mut egui_context: ResMut<EguiContext>,
    settings: ResMut<AssetBrowserSettings>,
    root_directory: ResMut<AssetDirectory>,
    currently_selected_directory: Res<SelectedDirectory>,
    editor_assets: Res<EditorAssets>,
    mut select_directory_event_writer: EventWriter<EnterDirectoryCommand>,
) {
    let ctx = egui_context.ctx_mut();
    let current_style = (*ctx.style()).clone();
    let mut new_style = current_style.clone();
    new_style.visuals.button_frame = false;
    ctx.set_style(new_style);

    let bottom_panel = TopBottomPanel::bottom("ContentBrowserPanel")
        .default_height(settings.default_height)
        .resizable(true);
    bottom_panel.show(ctx, |ui| {
        let tree_selection =
            draw_side_panel_tree_view(ui, &root_directory, settings.directory_hierarchy_widht);
        if let Some(Selection::Directory(selected_dir)) = tree_selection {
            let select_command = EnterDirectoryCommand {
                new_selected_directory: selected_dir,
                previous_selected_directory: currently_selected_directory.details.get_path(),
            };
            select_directory_event_writer.send(select_command);
        }

        let vertical_scroll_area = ScrollArea::vertical().auto_shrink([false, false]);
        vertical_scroll_area.show(ui, |ui| {
            if let Some(selected_path) = draw_assets(
                ui,
                settings.thumbnails_per_row,
                &currently_selected_directory.details,
                &editor_assets,
            ) {
                let select_command = EnterDirectoryCommand {
                    new_selected_directory: selected_path,
                    previous_selected_directory: currently_selected_directory.get_path(),
                };
                select_directory_event_writer.send(select_command);
            };
        })
    });
    ctx.set_style(current_style);
}

/// System for ResMut<SelectedDirectory> manipulation, with support for Undo and Redo events sent by
/// commands system
pub fn select_directory_system(
    mut normal_reader: EventReader<EnterDirectoryCommand>,
    mut undo_redo_reader: EventReader<UndoRedoCommandEvent>,
    mut selected_directory: ResMut<SelectedDirectory>,
    mut command_executed_writer: EventWriter<ExecuteCommandEvent>,
    root_directory: Res<AssetDirectory>,
) {
    for event in normal_reader.iter() {
        let new_selection_path = &event.new_selected_directory;
        if selected_directory.get_path() != *new_selection_path {
            println!("path: {:?}", new_selection_path);
            *selected_directory = root_directory
                .find_by_path(new_selection_path)
                .expect("Selected Directory should contain valid path")
                .into();

            command_executed_writer.send(ExecuteCommandEvent {
                inner: event.recreate(),
            });
        }
    }

    for undo_redo_event in undo_redo_reader.iter() {
        if undo_redo_event.cmd_type() == TypeId::of::<EnterDirectoryCommand>() {
            let selected_directory_command: &EnterDirectoryCommand =
                undo_redo_event.inner.as_any().downcast_ref().unwrap();
            let new_dir = match undo_redo_event.mode {
                CommandExecuteDirection::Redo => &selected_directory_command.new_selected_directory,
                CommandExecuteDirection::Undo => {
                    &selected_directory_command.previous_selected_directory
                }
            };
            *selected_directory = root_directory
                .find_by_path(new_dir)
                .expect("Undo/Redo should contain valid path")
                .into();
        }
    }
}
