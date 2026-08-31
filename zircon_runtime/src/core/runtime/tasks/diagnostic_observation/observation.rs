use std::sync::Arc;

use super::{TaskDiagnosticCursor, TaskDiagnosticIdentity};

pub const MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskDiagnosticKind {
    Cancelled,
    Panicked,
}

impl TaskDiagnosticKind {
    pub const fn severity(self) -> TaskDiagnosticSeverity {
        match self {
            Self::Cancelled => TaskDiagnosticSeverity::Warning,
            Self::Panicked => TaskDiagnosticSeverity::Error,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Panicked => "panicked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDiagnosticObservation {
    observation_sequence: u64,
    identity: TaskDiagnosticIdentity,
    kind: TaskDiagnosticKind,
    message: Arc<str>,
}

impl TaskDiagnosticObservation {
    pub(super) fn new(
        observation_sequence: u64,
        identity: TaskDiagnosticIdentity,
        kind: TaskDiagnosticKind,
        message: Arc<str>,
    ) -> Self {
        Self {
            observation_sequence,
            identity,
            kind,
            message: bounded_message(message),
        }
    }

    pub const fn observation_sequence(&self) -> u64 {
        self.observation_sequence
    }

    pub const fn identity(&self) -> TaskDiagnosticIdentity {
        self.identity
    }

    pub const fn kind(&self) -> TaskDiagnosticKind {
        self.kind
    }

    pub const fn severity(&self) -> TaskDiagnosticSeverity {
        self.kind.severity()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn next_cursor(&self) -> TaskDiagnosticCursor {
        TaskDiagnosticCursor::new(
            self.identity.scheduler_id(),
            self.observation_sequence.saturating_add(1),
        )
    }
}

fn bounded_message(message: Arc<str>) -> Arc<str> {
    if message.len() <= MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES {
        return message;
    }

    let mut end = MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES;
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    Arc::from(&message[..end])
}
