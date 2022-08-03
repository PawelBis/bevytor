use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_egui::egui::Event;

pub trait Command: Send + Sync + 'static {
    fn recreate(&self) -> Box<dyn Command>;
}

#[derive(Clone, Copy, Debug)]
pub enum CommandExecuteMode {
    Undo,
    Redo,
}

pub struct CommandExecutedEvent {
    pub inner: Box<dyn Command>,
    pub mode: CommandExecuteMode,
}

impl CommandExecutedEvent {
    pub fn consume(&self) -> Box<dyn Command> {
        self.inner.recreate()
    }

    pub fn get_direction(&self) -> CommandExecuteMode {
        self.mode
    }
}

pub struct CommandQueue {
    pub items: Vec<Box<dyn Command>>,
    pub pointer: Option<usize>,
}

impl CommandQueue {
    pub fn insert(&mut self, command: Box<dyn Command>) {
        if self.items.is_empty() {
            self.items.push(command);
            self.pointer = Some(self.items.len() - 1);
            return;
        }

        match self.pointer {
            Some(ptr) => {
                if ptr < self.items.len() - 1 {
                    self.items.truncate(ptr + 1);
                }
                self.items.push(command);
                self.pointer = Some(self.items.len() - 1);
            },
            None => {
                self.items.clear();
                self.insert(command);
            }
        }
    }

    pub fn redo(&mut self, commands_writer: &mut EventWriter<CommandExecutedEvent>,) {
        let post_redo_index = match self.pointer {
            Some(ptr) => ptr + 1,
            None => 0
        };
        match self.items.get(post_redo_index) {
            Some(command) => {
                commands_writer.send(CommandExecutedEvent {
                    inner: command.recreate(),
                    mode: CommandExecuteMode::Redo,
                });
                self.pointer = Some(post_redo_index);
            }
            None => println!("Redo chain empty!"),
        };
    }

    pub fn undo(&mut self, commands_writer: &mut EventWriter<CommandExecutedEvent>, ) {
        match self.pointer {
            Some(ptr) => {
                if let Some(command) = self.items.get(ptr) {
                    commands_writer.send(CommandExecutedEvent {
                        inner: command.recreate(),
                        mode: CommandExecuteMode::Undo,
                    });
                } else {
                    println!("No more items in undo chain!");
                }
            }
            None => println!("No more items in undo chain!")
        }
    }
}

pub struct EditorCommandsPlugin;
impl Plugin for EditorCommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<CommandExecutedEvent>()
            .insert_resource(CommandQueue {
                items: Vec::new(),
                pointer: None,
            })
            .add_system(process_commands_system)
            .add_system(undo_redo_system);
    }
}

// Consider sorting incoming events by timestamp
fn process_commands_system(
    mut queue: ResMut<CommandQueue>,
    mut commands: EventReader<CommandExecutedEvent>,
) {
    for command in commands.iter() {
        queue.insert(command.consume());
    }
}

fn undo_redo_system(
    keyboard: Res<Input<KeyCode>>,
    mut commands_writer: EventWriter<CommandExecutedEvent>,
    mut queue: ResMut<CommandQueue>,
) {
    if keyboard.pressed(KeyCode::LWin)
        && keyboard.just_pressed(KeyCode::Z) {
        queue.undo(&mut commands_writer);
    }

    if keyboard.pressed(KeyCode::LWin)
        && keyboard.just_pressed(KeyCode::Y) {
        queue.redo(&mut commands_writer);
    }
}

