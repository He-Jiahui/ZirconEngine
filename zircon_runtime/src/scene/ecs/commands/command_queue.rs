use std::fmt;

use crate::scene::{EntityId, World};

use super::{Command, ErasedCommand};

type QueuedCommand = Box<dyn ErasedCommand>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeferredCommandOperation {
    Spawn,
    Insert,
    InsertBundle,
    Remove,
    Despawn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredCommandError {
    operation: DeferredCommandOperation,
    entity: EntityId,
    message: String,
}

impl DeferredCommandError {
    pub fn new(
        operation: DeferredCommandOperation,
        entity: EntityId,
        message: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            entity,
            message: message.into(),
        }
    }

    pub fn operation(&self) -> DeferredCommandOperation {
        self.operation
    }

    pub fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeferredCommandReport {
    applied_count: usize,
    errors: Vec<DeferredCommandError>,
}

impl DeferredCommandReport {
    pub fn new(applied_count: usize, errors: Vec<DeferredCommandError>) -> Self {
        Self {
            applied_count,
            errors,
        }
    }

    pub fn applied_count(&self) -> usize {
        self.applied_count
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &[DeferredCommandError] {
        &self.errors
    }
}

#[derive(Default)]
pub struct CommandQueue {
    commands: Vec<QueuedCommand>,
}

impl CommandQueue {
    pub fn push(&mut self, command: impl Command) {
        self.commands.push(Box::new(command));
    }

    pub fn apply(&mut self, world: &mut World) -> DeferredCommandReport {
        let commands = std::mem::take(&mut self.commands);
        let applied_count = commands.len();
        world.clear_deferred_command_errors();
        for command in commands {
            command.apply_boxed(world);
        }
        DeferredCommandReport::new(applied_count, world.take_deferred_command_errors())
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
