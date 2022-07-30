use bevy::prelude::*;
use bevy_egui::egui::TextureId;
use bevy_egui::EguiContext;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTENSIONS: &[&str] = &["png"];

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app//.insert_resource(GameAssets::default())
            .add_startup_system(load_assets_system);
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum AssetType {
    Image(ImageAssetDescriptor),
}

impl AssetType {
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

// TODO: Change to generic treelike type
#[derive(Debug)]
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

    /// Checks if child is indeed child of this directory or any directory underneath
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

    fn try_insert_directory(&mut self, potential_child: AssetDirectory) -> Result<(), AssetDirectory> {
        if self.path == potential_child.path.parent().unwrap() {
            self.children_directories.push(potential_child);
            return Ok(());
        }

        let mut pot_child = potential_child;
        for child in self.children_directories.iter_mut() {
            match child.try_insert_directory(pot_child) {
                Ok(_) => return Ok(()),
                Err(err_child) => {
                    pot_child = err_child;
                }
            }
        }

        Err(pot_child)
    }

    fn try_insert_asset(&mut self, potential_child: AssetType) -> Result<(), AssetType> {
        if self.path == potential_child.path().parent().unwrap() {
            self.assets.push(potential_child);
            return Ok(());
        }

        let mut pot_child = potential_child;
        for child in self.children_directories.iter_mut() {
            match child.try_insert_asset(pot_child) {
                Ok(_) => return Ok(()),
                Err(err_child) => {
                    pot_child = err_child;
                }
            }
        }

        Err(pot_child)
    }
}

pub fn load_assets_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    const ASSET_DIRECTORY_NAME: &str = "assets";
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

    let path = &root.path;
    let files_count = root.assets.len();
    let dir_count = root.children_directories.len();
    println!("Directory {:?} contains {} children dir and {} assets!", path, dir_count, files_count);
    for dir in root.children_directories.iter() {
        let path = &dir.path;
        let files_count = dir.assets.len();
        let dir_count = dir.children_directories.len();
        println!("Directory {:?} contains {} children dir and {} assets!", path, dir_count, files_count);
    }
    commands.insert_resource(root);
}