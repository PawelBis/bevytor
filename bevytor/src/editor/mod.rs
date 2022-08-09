use assets::asset_loader::AssetLoaderPlugin;
use bevy::app::{Plugin, PluginGroup, PluginGroupBuilder};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy::scene::Scene;
use bevy_egui::EguiPlugin;
use crate::editor::commands::*;
use crate::editor::assets::asset_loader::*;
use crate::editor::ui::asset_browser::*;
use crate::editor::scene::scene_picker::*;
use ui::asset_browser::AssetBrowserPlugin;
use std::env;

pub mod assets;
pub mod commands;
pub mod scene;
pub mod ui;

fn run_if_post_initializing_assets(
    mut editor_state: ResMut<EditorStateLabel>
) -> ShouldRun {
    if *editor_state == EditorStateLabel::PostInitializingAssets {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_post_initializing_assets_dbg(
    mut editor_state: ResMut<EditorStateLabel>
) -> ShouldRun {
    println!("Checking the pi state");
    if *editor_state == EditorStateLabel::PostInitializingAssets {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

/// Label used for labeling editor dependent systems
/// EditorStateLabel::InitializingAssets - At this stage both editor and game assets are being
///                                         initialized and are not available
/// EditorStateLabel::PostInitializingAssets - At this stage both editor and game assets are available
#[derive(Debug, Clone, Eq, PartialEq, Hash, SystemLabel)]
pub enum EditorStateLabel {
    InitializingAssets,
    PostInitializingAssets,
}

/// This plugin group contains all the editor plugins and its dependencies, resulting in "complete" editor ui
pub struct EditorPlugins;
impl PluginGroup for EditorPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(EguiPlugin)
            .add(AssetLoaderPlugin)
            .add(EditorCommandsPlugin)
            .add(AssetBrowserPlugin)
            .add(ScenePickerPlugin);
    }
}

/// Plugin segregation was faulty, lets bind everything in one plugin
pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin);
        // Setup AssetLoaderPlugin
        //
        // It is really important to prepare AssetDirectory resource here and not in startup system,
        // as resources inserted through Commands are first available in the next frame
        const ASSET_DIRECTORY_NAME: &str = "assets";
        // TODO: Game asset directory should be editable per editor project
        const GAME_DIRECTORY_NAME: &str = "game";
        let asset_dir = env::current_dir()
            .unwrap()
            .join(GAME_DIRECTORY_NAME)
            .join(ASSET_DIRECTORY_NAME);
        let root = AssetDirectory::new(asset_dir.clone());

        app
            .insert_resource(root)
            .insert_resource(EditorAssets::default())
            .add_startup_system_set(
                SystemSet::new()
                    .with_system(load_editor_assets_system)
                    .label(EditorStateLabel::InitializingAssets)
                    .with_system(load_assets_system)
                    .label(EditorStateLabel::InitializingAssets)
            );

        // Setup EditorCommandsPlugin
        app.add_event::<CommandExecutedEvent>()
            .add_event::<UndoRedoCommandEvent>()
            .insert_resource(CommandQueue {
                items: Vec::new(),
                pointer: None,
            })
            .add_system(process_commands_system)
            .add_system(undo_redo_system);

        // Setup AssetBrowserPlugin
        app.add_event::<EnterDirectoryCommand>()
            .insert_resource(AssetBrowserSettings::default())
            .insert_resource(SelectedDirectory::default())
            .add_startup_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_post_initializing_assets)
                    .with_system(selection_setup)
                    .after(EditorStateLabel::InitializingAssets)
            )
            .add_system_set(
                SystemSet::new()
                    //.with_run_criteria(run_if_post_initializing_assets)
                    .with_system(asset_browser_system)
                    .with_system(select_directory_system)
                    .after(EditorStateLabel::InitializingAssets)
            );

        // Setup ScenePickerPlugin
        app
            .insert_resource(MainScene::default())
            //.add_startup_system_set(
            //    SystemSet::new()
            //        //.with_run_criteria(run_if_post_initializing_assets_dbg)
            //        .with_system(create_scene_system)
            //        .after(EditorStateLabel::InitializingAssets)
            //)
            .add_system_set(
                SystemSet::new()
                    //.with_run_criteria(run_if_post_initializing_assets)
                    .with_system(scene_picker_system)
                    .with_system(create_scene_system)
                    .after(EditorStateLabel::InitializingAssets)
            );
    }
}
