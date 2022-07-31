use bevy::prelude::*;
use bevy_egui::egui::{TextureId, ColorImage};
use bevy_egui::EguiContext;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTENSIONS: &[&str] = &["png"];

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(load_editor_assets_system)
            .add_startup_system(load_assets_system);
    }
}

#[derive(Debug, Clone)]
pub struct ImageAssetDescriptor {
    /// Name of the asset, with extension
    pub name: OsString,
    /// Path relative to assets directory
    pub path: PathBuf,
    /// Bevy asset handle
    pub bevy_handle: Handle<Image>,
    /// Egui texture id
    pub egui_texture_id: TextureId,
}

#[derive(Debug, Clone)]
pub enum AssetType {
    Image(ImageAssetDescriptor),
}

impl AssetType {
    /// Tries to create asset from given path.
    /// TODO: Make extension detection more sophisticated
    fn try_create(
        path: &Path,
        asset_server: &AssetServer,
        egui_context: &mut EguiContext
    ) -> Option<Self> {
        let is_image = IMAGE_EXTENSIONS
            .into_iter()
            .any(|ext| path.file_name().unwrap().to_string_lossy().ends_with(ext));
        if !is_image {
            return None;
        }

        let bevy_handle = asset_server.load(path.clone());
        let egui_texture_id = egui_context.add_image(bevy_handle.as_weak());

        Some(Self::Image(ImageAssetDescriptor {
            name: path.file_name().unwrap().to_os_string(),
            path: path.to_path_buf(),
            bevy_handle,
            egui_texture_id,
        }))
    }

    fn path(&self) -> PathBuf {
        match self {
            AssetType::Image(i) => i.path.clone()
        }
    }
}

/// Stores data about directory and supported assets contained within it
/// TODO: Change to generic treelike type
#[derive(Debug, Clone)]
pub struct AssetDirectory {
    /// Name of the directory
    pub name: OsString,
    /// Path relative to assets directory
    pub path: PathBuf,
    /// List children directories
    pub children_directories: Vec<AssetDirectory>,
    /// List of assets located in this directory
    pub assets: Vec<AssetType>,
}

impl AssetDirectory {
    fn new(path: PathBuf) -> Self {
        Self {
            name: path.file_name().unwrap().to_os_string(),
            path,
            children_directories: Vec::new(),
            assets: Vec::new(),
        }
    }

    /// Checks if path is supported asset or directory and adds it to proper category
    /// and inserts it if so
    fn try_insert(
        &mut self,
        path: &Path,
        asset_server: &AssetServer,
        mut egui_context: &mut EguiContext
    ) -> bool {
        if path.metadata().unwrap().is_dir() {
            self.try_insert_directory(AssetDirectory::new(path.to_path_buf())).is_ok()
        } else if let Some(asset) = AssetType::try_create(path, asset_server, &mut egui_context) {
            self.try_insert_asset(asset).is_ok()
        } else {
            false
        }
    }

    /// Checks if given directory is child of any directory in the hierarchy and
    /// stores it if it's true. Returns given directory back in case of error
    fn try_insert_directory(&mut self, potential_child: AssetDirectory) -> Result<(), AssetDirectory> {
        if self.path == potential_child.path.parent().unwrap() {
            self.children_directories.push(potential_child);
            return Ok(());
        }

        let mut potential_child = potential_child;
        for child in self.children_directories.iter_mut() {
            match child.try_insert_directory(potential_child) {
                Ok(_) => return Ok(()),
                Err(err_child) => {
                    potential_child = err_child;
                }
            }
        }
        Err(potential_child)
    }

    /// Checks if an asset is child of any of the directories in the hierarchy
    /// TODO: Both try_insert_functions are basically the same and could be reduced to one fn
    fn try_insert_asset(&mut self, potential_child: AssetType) -> Result<(), AssetType> {
        if self.path == potential_child.path().parent().unwrap() {
            self.assets.push(potential_child);
            return Ok(());
        }

        let mut potential_child = potential_child;
        for child in self.children_directories.iter_mut() {
            match child.try_insert_asset(potential_child) {
                Ok(_) => return Ok(()),
                Err(err_child) => {
                    potential_child = err_child;
                }
            }
        }

        Err(potential_child)
    }

    /// Find currently selected directory
    pub fn find_by_predicate (
        &self,
        pred: impl Fn(&AssetDirectory) -> bool
    ) -> Option<&AssetDirectory> {
        if pred(self) {
            return Some(&self);
        } else {
            for child in self.children_directories.iter() {
                if pred(child) {
                    return Some(&child);
                }
            }
        }

        None
    }

    pub fn find_by_predicate_mut(
        &mut self,
        pred: impl Fn(&mut AssetDirectory) -> bool
    ) -> Option<&mut AssetDirectory> {
        if pred(self) {
            return Some(self);
        } else {
            for child in self.children_directories.iter_mut() {
                if pred(child) {
                    return Some(child);
                }
            }
        }

        None
    }
}

pub struct EditorAssets {
    pub directory_icon: TextureId,
}

fn load_editor_assets_system(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut asset_server: ResMut<AssetServer>,
) {
    println!("Loading editor assets");
    const EDITOR_ASSETS_DIRECTORY: &str = "assets";
    const EDITOR_NAME: &str = "bevytor";
    let editor_assets_dir = env::current_dir()
        .unwrap()
        .join(EDITOR_NAME)
        .join(EDITOR_ASSETS_DIRECTORY);
    let bevy_handle: Handle<Image> = asset_server.load(
        editor_assets_dir
            .join("folder.png")
            .as_path()
    );
    let editor_assets = EditorAssets {
        directory_icon: egui_context
            .add_image(bevy_handle),
    };
    commands.insert_resource(editor_assets);
}


pub fn load_assets_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    println!("Loading assets");
    const ASSET_DIRECTORY_NAME: &str = "assets";
    // TODO: Game asset directory should be editable per editor project
    const GAME_DIRECTORY_NAME: &str = "game";
    let asset_dir = env::current_dir()
        .unwrap()
        .join(GAME_DIRECTORY_NAME)
        .join(ASSET_DIRECTORY_NAME);

    let mut root = AssetDirectory::new(asset_dir.clone());
    for entry in WalkDir::new(asset_dir.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        root.try_insert(entry.path(), &asset_server, &mut egui_ctx);
    }

    commands.insert_resource(root);
}