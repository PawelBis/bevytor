use std::any::Any;
use std::fmt::{Display, Formatter};
use bevy::prelude::*;

pub trait CommandAny: Command + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> CommandAny for T
    where T: Command + Any
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Command: Send + Sync + 'static {
    fn recreate(&self) -> Box<dyn CommandAny>;
    fn command_type(&self) -> &str;
}

#[derive(Clone, Copy, Debug)]
pub enum CommandExecuteMode {
    Undo,
    Redo,
}

pub struct CommandExecutedEvent {
    pub inner: Box<dyn CommandAny>,
}
impl CommandExecutedEvent {
    pub fn consume(&self) -> Box<dyn CommandAny> {
        self.inner.recreate()
    }
}

pub struct UndoRedoCommandEvent {
    pub inner: Box<dyn CommandAny>,
    pub mode: CommandExecuteMode,
}
impl UndoRedoCommandEvent {
    pub fn consume(&self) -> Box<dyn CommandAny> { self.inner.recreate() }
    pub fn command_type(&self) -> &str { self.inner.command_type() }
}

pub struct CommandQueue {
    pub items: Vec<Box<dyn CommandAny>>,
    pub pointer: Option<usize>,
}
impl Display for CommandQueue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CommandQueue lenght: {}, pointer position: {:?}", self.items.len(), self.pointer)
    }
}

impl CommandQueue {
    pub fn insert(&mut self, command: Box<dyn CommandAny>) {
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
        println!("{}", self);
    }

    pub fn redo(&mut self, commands_writer: &mut EventWriter<UndoRedoCommandEvent>,) {
        let post_redo_index = match self.pointer {
            Some(ptr) => ptr + 1,
            None => 0
        };
        match self.items.get(post_redo_index) {
            Some(command) => {
                commands_writer.send(UndoRedoCommandEvent {
                    inner: command.recreate(),
                    mode: CommandExecuteMode::Redo,
                });
                self.pointer = Some(post_redo_index);
            }
            None => println!("Redo chain empty!"),
        };
        println!("{}", self);
    }

    pub fn undo(&mut self, commands_writer: &mut EventWriter<UndoRedoCommandEvent>, ) {
        match self.pointer {
            Some(ptr) => {
                if let Some(command) = self.items.get(ptr) {
                    commands_writer.send(UndoRedoCommandEvent {
                        inner: command.recreate(),
                        mode: CommandExecuteMode::Undo,
                    });
                    self.pointer = if ptr > 0 {
                        Some(ptr - 1)
                    } else {
                        None
                    };
                } else {
                    println!("No more items in undo chain!");
                }
            }
            None => println!("No more items in undo chain!")
        }
        println!("{}", self);
    }
}

pub struct EditorCommandsPlugin;
impl Plugin for EditorCommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<CommandExecutedEvent>()
            .add_event::<UndoRedoCommandEvent>()
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
    mut commands_writer: EventWriter<UndoRedoCommandEvent>,
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

