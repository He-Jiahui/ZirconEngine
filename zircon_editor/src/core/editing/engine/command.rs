use std::any::Any;
use std::error::Error;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::HistoryContextId;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SelectionSnapshot(serde_json::Value);

impl SelectionSnapshot {
    pub fn from_json(value: serde_json::Value) -> Self {
        Self(value)
    }

    pub fn as_json(&self) -> &serde_json::Value {
        &self.0
    }
}

pub trait EditContext: Any + Send {
    fn selection_snapshot(&self) -> SelectionSnapshot;

    fn restore_selection(&mut self, snapshot: &SelectionSnapshot) -> Result<(), EditCommandError>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    Reject,
    Merged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandEffect {
    Unchanged,
    Applied,
}

#[derive(Debug, Error)]
#[error("edit command failed with effect {effect:?}")]
pub struct CommandExecutionError {
    pub effect: CommandEffect,
    #[source]
    pub source: EditCommandError,
}

impl CommandExecutionError {
    pub fn unchanged(source: EditCommandError) -> Self {
        Self {
            effect: CommandEffect::Unchanged,
            source,
        }
    }

    pub fn applied(source: EditCommandError) -> Self {
        Self {
            effect: CommandEffect::Applied,
            source,
        }
    }
}

pub trait EditCommand: Any + Send {
    fn label(&self) -> &str;

    fn is_significant(&self) -> bool {
        true
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError>;

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError>;

    fn finalize(&mut self, _context: &mut dyn EditContext) {}

    fn try_merge(&mut self, _next: &dyn EditCommand) -> MergeOutcome {
        MergeOutcome::Reject
    }

    fn serialize_journal(&self) -> Option<serde_json::Value> {
        None
    }

    fn as_any(&self) -> &dyn Any;
}

pub type CommandBox = Box<dyn EditCommand>;

#[derive(Debug, Error)]
pub enum EditCommandError {
    #[error("edit target is missing: {target}")]
    TargetMissing { target: String },
    #[error("edit invariant was violated: {invariant}")]
    InvariantViolation { invariant: &'static str },
    #[error("reflection edit failed")]
    ReflectError {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("external edit effect failed")]
    ExternalEffect {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("edit context has the wrong concrete type; expected {expected}")]
    ContextTypeMismatch { expected: &'static str },
    #[error("history capacity must be greater than zero")]
    InvalidHistoryCapacity,
    #[error("cannot nest history {requested:?} inside active history {active:?}")]
    CrossContextNested {
        active: HistoryContextId,
        requested: HistoryContextId,
    },
    #[error("transaction scope is no longer active")]
    ScopeClosed,
    #[error("editor transaction engine is busy with {active}; cannot start {requested}")]
    EngineBusy {
        active: &'static str,
        requested: &'static str,
    },
    #[error("editor transaction engine is faulted during {operation}")]
    EngineFaulted { operation: &'static str },
    #[error("transaction identifier space is exhausted")]
    TransactionIdExhausted,
    #[error("command rollback failed after an earlier command error")]
    RollbackFailed {
        command_error: Box<EditCommandError>,
        #[source]
        rollback_error: Box<EditCommandError>,
    },
}
