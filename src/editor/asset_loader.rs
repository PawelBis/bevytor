use bevy::prelude::*;
use bevy_egui::egui::TextureId;
use bevy_egui::EguiContext;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

const IMAGE_EXTENSIONS: &[&str] = &["png"];

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default())
            .add_startup_system(load_assets_system);
    }
}

pub struct ImageAsset {
    pub handle: Handle<Image>,
    pub egui_texture_id: TextureId,
}

#[derive(Default)]
pub struct GameAssets {
    pub images: HashMap<PathBuf, ImageAsset>,
}

pub fn load_assets_system(
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut assets: ResMut<GameAssets>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    let asset_dir = env::current_dir().unwrap().join("assets");
    for entry in WalkDir::new(asset_dir.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let filename = entry.file_name().to_string_lossy();
        let is_image = IMAGE_EXTENSIONS
            .into_iter()
            .any(|ext| filename.ends_with(ext));
        if is_image {
            let relative_path = entry.path().strip_prefix(asset_dir.clone()).unwrap();
            let handle = asset_server.load(relative_path);
            let size = Vec2::new(150.0, 150.0);
            let egui_texture_id = egui_ctx.add_image(handle.as_weak());
            assets.images.insert(
                relative_path.into(),
                ImageAsset {
                    handle,
                    egui_texture_id,
                },
            );
        }
    }
    info!("Asset loader found {} images!", assets.images.len());
}
