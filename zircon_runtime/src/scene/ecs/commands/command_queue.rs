use std::fmt;

use crate::scene::World;

use super::{Command, ErasedCommand};

type QueuedCommand = Box<dyn ErasedCommand>;

#[derive(Default)]
pub struct CommandQueue {
    commands: Vec<QueuedCommand>,
}

impl CommandQueue {
    pub fn push(&mut self, command: impl Command) {
        self.commands.push(Box::new(command));
    }

    pub fn apply(&mut self, world: &mut World) {
        for command in std::mem::take(&mut self.commands) {
            command.apply_boxed(world);
        }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl fmt::Debug for CommandQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandQueue")
            .field("len", &self.commands.len())
            .finish()
    }
}

impl Clone for CommandQueue {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for CommandQueue {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
