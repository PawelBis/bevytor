use bevy::prelude::*;
use std::any::Any;
use std::fmt::{Display, Formatter};

/// Auto trait enabling command downcasting
pub trait CommandAny: Command + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> CommandAny for T
where
    T: Command + Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Command trait used by undo/redo chain
pub trait Command: Send + Sync + 'static {
    fn recreate(&self) -> Box<dyn CommandAny>;
    fn command_type(&self) -> &str;
}

/// Used by undo/redo chain to specify how the command should be repeated
#[derive(Clone, Copy, Debug)]
pub enum CommandExecuteDirection {
    Undo,
    Redo,
}

/// Systems relying on undo/redo should send all their executed commands through this Event
pub struct CommandExecutedEvent {
    pub inner: Box<dyn CommandAny>,
}
impl CommandExecutedEvent {
    pub fn consume(&self) -> Box<dyn CommandAny> {
        self.inner.recreate()
    }
}

/// Undo/Redo system resends commands retrieved through CommandExecutedEvents
pub struct UndoRedoCommandEvent {
    pub inner: Box<dyn CommandAny>,
    pub mode: CommandExecuteDirection,
}
impl UndoRedoCommandEvent {
    pub fn consume(&self) -> Box<dyn CommandAny> {
        self.inner.recreate()
    }
    pub fn command_type(&self) -> &str {
        self.inner.command_type()
    }
}

/// Resource for Undo/Redo chain manipulation
pub struct CommandQueue {
    /// Commands stored in the chain
    pub items: Vec<Box<dyn CommandAny>>,
    /// Index of command that will be "undoed" after pressing Ctrl+z / Cmd+z
    pub pointer: Option<usize>,
}
impl Display for CommandQueue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CommandQueue lenght: {}, pointer position: {:?}",
            self.items.len(),
            self.pointer
        )
    }
}

impl CommandQueue {
    /// Insert incoming command at the end of the Undo/Redo chain.
    /// All Commands stored after the pointer (with index > pointer) will be removed
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
            }
            None => {
                self.items.clear();
                self.insert(command);
            }
        }
        println!("{}", self);
    }

    /// Send UndoRedoCommandEvent with CommandExecuteDirection::Redo and increase the pointer
    pub fn redo(&mut self, commands_writer: &mut EventWriter<UndoRedoCommandEvent>) {
        let post_redo_index = match self.pointer {
            Some(ptr) => ptr + 1,
            None => 0,
        };
        match self.items.get(post_redo_index) {
            Some(command) => {
                commands_writer.send(UndoRedoCommandEvent {
                    inner: command.recreate(),
                    mode: CommandExecuteDirection::Redo,
                });
                self.pointer = Some(post_redo_index);
            }
            None => println!("Redo chain empty!"),
        };
        println!("{}", self);
    }

    /// Send UndoRedoCommandEvent with CommandExecuteDirection::Undo and decrease the pointer
    pub fn undo(&mut self, commands_writer: &mut EventWriter<UndoRedoCommandEvent>) {
        match self.pointer {
            Some(ptr) => {
                if let Some(command) = self.items.get(ptr) {
                    commands_writer.send(UndoRedoCommandEvent {
                        inner: command.recreate(),
                        mode: CommandExecuteDirection::Undo,
                    });
                    self.pointer = if ptr > 0 { Some(ptr - 1) } else { None };
                } else {
                    println!("No more items in undo chain!");
                }
            }
            None => println!("No more items in undo chain!"),
        }
        println!("{}", self);
    }
}

pub struct EditorCommandsPlugin;
impl Plugin for EditorCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandExecutedEvent>()
            .add_event::<UndoRedoCommandEvent>()
            .insert_resource(CommandQueue {
                items: Vec::new(),
                pointer: None,
            })
            .add_system(process_commands_system)
            .add_system(undo_redo_system);
    }
}

/// Naive reading of CommandExecutedEvents and moving them to the CommandQueue.
/// Consider sorting the events by the timestamp
fn process_commands_system(
    mut queue: ResMut<CommandQueue>,
    mut commands: EventReader<CommandExecutedEvent>,
) {
    for command in commands.iter() {
        queue.insert(command.consume());
    }
}

/// System reading keyboard input and producing Undo and Redo commands
fn undo_redo_system(
    keyboard: Res<Input<KeyCode>>,
    mut commands_writer: EventWriter<UndoRedoCommandEvent>,
    mut queue: ResMut<CommandQueue>,
) {
    if keyboard.pressed(KeyCode::LWin) && keyboard.just_pressed(KeyCode::Z) {
        queue.undo(&mut commands_writer);
    }

    if keyboard.pressed(KeyCode::LWin) && keyboard.just_pressed(KeyCode::Y) {
        queue.redo(&mut commands_writer);
    }
}
