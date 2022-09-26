use std::{any::TypeId, fs::File, io::Write};

use crate::editor::assets::asset_loader::SceneAssetDescriptor;
use bevy::{prelude::*, reflect::TypeRegistry, tasks::IoTaskPool};
use bevy_egui::{
    egui::{self, Align2, Ui, Window},
    EguiContext,
};

use super::{
    assets::asset_loader::AssetDescriptor,
    commands::{Command, CommandAny, UndoRedoCommandEvent, ExecuteCommandEvent, CommandExecuteDirection},
    ShowCreateSceneWidgetContext,
};

pub struct EditorScenePlugin;
impl Plugin for EditorScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedScene::default())
            .add_system(open_scene_system)
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

// Returns true if scene was saved
fn create_and_save_scene(
    path: String,
    parent: String,
) -> bool {
    let world = World::new();
    let type_registry = TypeRegistry::default();
    let scene = DynamicScene::from_world(&world, &type_registry);
    let serialized_scene = scene.serialize_ron(&type_registry).unwrap();
    if !std::path::Path::new(&path).exists() {
        false
    } else {
        std::fs::create_dir_all(parent).unwrap();
        File::create(path).and_then(|mut file| file.write(serialized_scene.as_bytes())).is_ok()
    }
}

pub fn create_scene_system(
    mut create_scene_command_reader: EventReader<CreateSceneCommand>,
    mut undo_redo_command_reader: EventReader<UndoRedoCommandEvent>,
    mut execute_command_writer: EventWriter<ExecuteCommandEvent>,
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
        if create_and_save_scene(new_scene_path, new_scene_parent) {
            execute_command_writer.send(
                ExecuteCommandEvent { inner: create_scene.recreate() }
            );
        }
    }

    for undo_redo_event in undo_redo_command_reader.iter() {
        if undo_redo_event.cmd_type() != TypeId::of::<CreateSceneCommand>() {
            continue;
        }

        let create_scene_event: &CreateSceneCommand = undo_redo_event.inner.as_any().downcast_ref().unwrap();
        match undo_redo_event.mode {
            CommandExecuteDirection::Undo => {
                if let Some(scene_descriptor) = &create_scene_event.scene {
                    match std::fs::remove_file(scene_descriptor.get_path()) {
                        Ok(_) => (),
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::PermissionDenied => {
                                error!("Failed to delete file: {:?}", scene_descriptor.get_path());
                            },
                            _ => (),
                        },
                    };
                }
            },
            CommandExecuteDirection::Redo => {
                let recreate_scene: &CreateSceneCommand = undo_redo_event.inner.as_any().downcast_ref().unwrap();
                let path = recreate_scene.scene.clone().unwrap().get_path();
                create_and_save_scene(path.to_str().unwrap().to_string(), path.parent().unwrap().to_str().unwrap().to_string());
            },
        }
    }
}

pub struct OpenSceneCommand {
    next: Option<SceneAssetDescriptor>,
    previous: Option<SceneAssetDescriptor>,
}

impl Command for OpenSceneCommand {
    fn recreate(&self) -> Box<dyn CommandAny> {
        Box::new(Self {
            next: self.next.clone(),
            previous: self.previous.clone(),
        })
    }

    fn command_type(&self) -> TypeId {
        TypeId::of::<OpenSceneCommand>()
    }
}

pub fn open_scene_system(mut select_scene_reader: EventReader<OpenSceneCommand>) {
    for _command in select_scene_reader.iter() {
        println!("Selected scene bitch");
    }
}
