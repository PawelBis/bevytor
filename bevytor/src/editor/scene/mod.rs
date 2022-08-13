use std::any::TypeId;

use bevy::{app::{App, Plugin}, prelude::EventReader};
use crate::editor::assets::asset_loader::SceneAssetDescriptor;

use super::commands::{Command, CommandAny};

pub struct EditorScenePlugin;
impl Plugin for EditorScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SelectedScene::default());
    }
}

#[derive(Default, Clone)]
pub struct SelectedScene {
    descriptior: Option<SceneAssetDescriptor>,
}

pub struct SelectSceneCommand {
    next: Option<SceneAssetDescriptor>,
    previous: Option<SceneAssetDescriptor>,
}

impl Command for SelectSceneCommand {
    fn recreate(&self) -> Box<dyn CommandAny> {
        Box::new(Self {
            next: self.next.clone(),
            previous: self.previous.clone(),
        })
    }

    fn command_type(&self) -> TypeId {
        TypeId::of::<SelectSceneCommand>()
    }
}

pub fn select_scene_system(
    _selected_scene_reader: EventReader<SelectSceneCommand>,
) {

}

