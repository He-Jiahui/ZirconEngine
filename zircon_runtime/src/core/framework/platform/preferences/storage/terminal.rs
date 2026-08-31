use super::super::{PreferenceStorageErrorKind, PreferenceStorageOperation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferencePersistenceFailureProjection {
    kind: PreferenceStorageErrorKind,
    operation: PreferenceStorageOperation,
    backend: &'static str,
    detail: String,
}

impl PreferencePersistenceFailureProjection {
    pub(crate) fn new(
        kind: PreferenceStorageErrorKind,
        operation: PreferenceStorageOperation,
        backend: &'static str,
        detail: String,
    ) -> Self {
        Self {
            kind,
            operation,
            backend,
            detail,
        }
    }

    pub const fn kind(&self) -> PreferenceStorageErrorKind {
        self.kind
    }

    pub const fn operation(&self) -> PreferenceStorageOperation {
        self.operation
    }

    pub const fn backend(&self) -> &'static str {
        self.backend
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceMutationTerminal {
    Durable,
    Failed(PreferencePersistenceFailureProjection),
    DeadlineBeforeStart,
    CancelledBeforeStart,
    Superseded { successor: u64 },
    Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceTicketWaitResult {
    Terminal(PreferenceMutationTerminal),
    ObserverTimedOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceMutationCancelError {
    AlreadyStarted,
    FencePinned,
    WrongAuthority,
}
