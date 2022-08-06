use crate::editor::EditorStateLabel;
use bevy::prelude::*;
use bevy_egui::egui::TextureId;
use bevy_egui::EguiContext;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTENSIONS: &[&str] = &["png", "hdr"];

/// AssetLoaderPlugin iterates over bevys "asset" folder creating directory hierarchy
/// and loading all supported assets
pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
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

        app.insert_resource(root)
            .add_startup_system(
                load_editor_assets_system
                    .label(EditorStateLabel::InitializingAssets)
                    .before(EditorStateLabel::PostInitializingAssets),
            )
            .add_startup_system(
                load_assets_system
                    .label(EditorStateLabel::InitializingAssets)
                    .before(EditorStateLabel::PostInitializingAssets),
            );
    }
}

pub trait AssetDescriptor {
    fn get_name(&self) -> String;
    fn get_path(&self) -> PathBuf;
}

/// Bevytor image descriptor, contains handlers required for rendering it both in egui and bevy.
/// egui is only using WEAK handler of the texture and only bevy is storing the image
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

impl AssetDescriptor for ImageAssetDescriptor {
    fn get_name(&self) -> String {
        self.name.to_string_lossy().to_string()
    }

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

/// All asset types currently supported in Bevytor. This enum will grow over time and at some
/// point it will be moved to separate module
#[derive(Debug, Clone)]
pub enum AssetType {
    Image(ImageAssetDescriptor),
}

impl AssetType {
    /// Try create an asset from given path. Naive implementation
    /// TODO: Make extension detection more sophisticated
    fn try_create(
        path: &Path,
        asset_server: &AssetServer,
        egui_context: &mut EguiContext,
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

    pub fn get_path(&self) -> PathBuf {
        match self {
            AssetType::Image(i) => i.get_path(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            AssetType::Image(i) => i.name.to_string_lossy().to_string(),
        }
    }
}

/// Aggregates data about the asset directory and hierarchy in tree like container
/// TODO: Change to generic treelike type
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
impl PartialEq<Self> for AssetDirectory {
    fn eq(&self, other: &Self) -> bool {
        other.path == self.path
    }
}
impl Eq for AssetDirectory {}

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
        mut egui_context: &mut EguiContext,
    ) -> bool {
        if path.metadata().unwrap().is_dir() {
            self.try_insert_directory(AssetDirectory::new(path.to_path_buf()))
                .is_ok()
        } else if let Some(asset) = AssetType::try_create(path, asset_server, &mut egui_context) {
            self.try_insert_asset(asset).is_ok()
        } else {
            false
        }
    }

    /// Checks if given directory is child of any directory in the hierarchy and
    /// stores it if it's true. Returns given directory back in case of error
    fn try_insert_directory(
        &mut self,
        potential_child: AssetDirectory,
    ) -> Result<(), AssetDirectory> {
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
        if self.path == potential_child.get_path().parent().unwrap() {
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

    /// Find directory that satisfies given predicate
    pub fn find_by_predicate(
        &self,
        pred: impl Fn(&AssetDirectory) -> bool,
    ) -> Option<&AssetDirectory> {
        if pred(self) {
            return Some(&self);
        } else {
            for child in self.children_directories.iter() {
                if pred(child) {
                    return Some(&child);
                } else {
                    return child.find_by_predicate(pred);
                }
            }
        }

        None
    }

    /// Find directory by path. Convenience fn using `find_by_predicate` underneath
    pub fn find_by_path(&self, path: &PathBuf) -> Option<&AssetDirectory> {
        self.find_by_predicate(|dir| dir.path == *path)
    }

    /// Find directory that satisfies given predicate
    pub fn find_by_predicate_mut(
        &mut self,
        pred: impl Fn(&mut AssetDirectory) -> bool,
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

    pub fn get_name(&self) -> String {
        self.name.to_string_lossy().to_string()
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

/// Special assets used by the editor. Right now editor assets are stored in bevytor crate root dir.
/// TODO: Provide config for specifying game assets directory and editor assets directory
pub struct EditorAssets {
    pub directory_icon: TextureId,
}

/// Load assets commonly used around the editor
/// TODO: Consider moving this system to build fn
pub fn load_editor_assets_system(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    asset_server: ResMut<AssetServer>,
) {
    println!("Loading editor assets");
    const EDITOR_ASSETS_DIRECTORY: &str = "assets";
    const EDITOR_NAME: &str = "bevytor";
    let editor_assets_dir = env::current_dir()
        .unwrap()
        .join(EDITOR_NAME)
        .join(EDITOR_ASSETS_DIRECTORY);
    let bevy_handle: Handle<Image> =
        asset_server.load(editor_assets_dir.join("folder.png").as_path());
    let editor_assets = EditorAssets {
        directory_icon: egui_context.add_image(bevy_handle),
    };
    commands.insert_resource(editor_assets);
}

/// Load assets stored in the game assets directory
/// TODO: Consider moving this system to build fn
pub fn load_assets_system(
    asset_server: Res<AssetServer>,
    mut egui_ctx: ResMut<EguiContext>,
    mut root: ResMut<AssetDirectory>,
) {
    println!("Loading assets");
    for entry in WalkDir::new(root.path.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        root.try_insert(entry.path(), &asset_server, &mut egui_ctx);
    }
}
