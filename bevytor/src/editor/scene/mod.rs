use std::{any::TypeId, fs::File, io::Write};

use crate::editor::assets::asset_loader::SceneAssetDescriptor;
use bevy::{prelude::*, reflect::TypeRegistry, tasks::IoTaskPool};
use bevy_egui::{
    egui::{self, Align2, Ui, Window},
    EguiContext,
};

use super::{
    assets::asset_loader::AssetDescriptor,
    commands::{Command, CommandAny, UndoRedoCommandEvent},
    ShowCreateSceneWidgetContext,
};

pub struct EditorScenePlugin;
impl Plugin for EditorScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedScene::default())
            .add_system(select_scene_system)
            .add_system(create_scene_system);
    }
}

#[derive(Default, Clone)]
pub struct SelectedScene {
    pub descriptor: Option<SceneAssetDescriptor>,
}

#[derive(Clone)]
pub struct CreateSceneCommand {
    pub scene: Option<SceneAssetDescriptor>,
}

impl CreateSceneCommand {
    pub fn widget(
        context: &mut EguiContext,
        widget_context: &mut ShowCreateSceneWidgetContext,
    ) -> Option<CreateSceneCommand> {
        let mut create_scene_command = None;
        let mut is_open = widget_context.show_widget;
        let window = Window::new("Create new scene")
            .open(&mut widget_context.show_widget)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, (0.0, 0.0));
        if is_open {
            window.show(context.ctx_mut(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut widget_context.scene_name);
                });
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        create_scene_command = Some(CreateSceneCommand {
                            scene: Some(SceneAssetDescriptor {
                                name: widget_context.scene_name.as_str().into(),
                                path: format!("game/assets/scenes/{}", widget_context.scene_name)
                                    .into(),
                            }),
                        });
                        is_open = false;
                    }
                    if ui.button("Cancel").clicked() {
                        is_open = false;
                    }
                })
            });
            if is_open == false {
                widget_context.show_widget = false;
            }
        };

        create_scene_command
    }
}

impl Command for CreateSceneCommand {
    fn recreate(&self) -> Box<dyn CommandAny> {
        Box::new(self.clone())
    }

    fn command_type(&self) -> TypeId {
        TypeId::of::<CreateSceneCommand>()
    }
}

pub fn create_scene_system(
    mut create_scene_command_reader: EventReader<CreateSceneCommand>,
    mut _undo_redo_command_reader: EventReader<UndoRedoCommandEvent>,
) {
    for create_scene in create_scene_command_reader.iter() {
        let (new_scene_path, new_scene_parent) = match &create_scene.scene {
            Some(scene_descriptor) => {
                let path = scene_descriptor
                    .get_path()
                    .as_path()
                    .to_str()
                    .unwrap()
                    .to_string();
                let parent = scene_descriptor
                    .get_path()
                    .as_path()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                (path, parent)
            }
            None => continue,
        };
        let world = World::new();
        let type_registry = TypeRegistry::default();
        let scene = DynamicScene::from_world(&world, &type_registry);
        let serialized_scene = scene.serialize_ron(&type_registry).unwrap();
        IoTaskPool::get()
            .spawn(async move {
                std::fs::create_dir_all(new_scene_parent).unwrap();
                if !std::path::Path::new(&new_scene_path).exists() {
                    match File::create(new_scene_path)
                        .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    {
                        Ok(_) => (),
                        Err(e) => error!("Failed to save scene: {e}"),
                    }
                }
            })
            .detach();
    }
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

pub fn select_scene_system(mut select_scene_reader: EventReader<SelectSceneCommand>) {
    for _command in select_scene_reader.iter() {
        println!("Selected scene bitch");
    }
}
