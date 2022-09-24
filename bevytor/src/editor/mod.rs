use assets::asset_loader::AssetLoaderPlugin;
use bevy::app::{Plugin, PluginGroup, PluginGroupBuilder};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use crate::editor::commands::*;
use crate::editor::assets::asset_loader::*;
use crate::editor::ui::asset_browser::*;
use crate::editor::scene::{EditorScenePlugin, SelectedScene, create_scene_system, CreateSceneCommand};
use ui::asset_browser::AssetBrowserPlugin;
use std::env;

pub mod assets;
pub mod commands;
pub mod scene;
pub mod ui;

fn run_if_post_initializing_assets(
    editor_state: Res<EditorStateLabel>
) -> ShouldRun {
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
            .add(EditorScenePlugin);
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
            .add_startup_system(load_editor_assets_system)
            .add_startup_system(load_assets_system.after(load_editor_assets_system));

        // Setup EditorCommandsPlugin
        app.add_event::<ExecuteCommandEvent>()
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
            .add_startup_system(selection_setup.after(load_assets_system))
            .add_system(asset_browser_system)
            .add_system(select_directory_system);

        // Setup ScenePickerPlugin
        app
            .insert_resource(SelectedScene::default())
            .add_event::<CreateSceneCommand>()
            .add_system(create_scene_system);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
