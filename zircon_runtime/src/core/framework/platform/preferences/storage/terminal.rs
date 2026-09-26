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

/// 一次已受理工作的终态；失败或截止前未启动仍可能留下可见但未持久化代际。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreferenceMutationTerminal {
    Durable,
    Failed(PreferencePersistenceFailureProjection),
    DeadlineBeforeStart,
    CancelledBeforeStart,
    Superseded { successor: u64 },
    Shutdown,
}

/// 等待者超时仅结束本次观察，不会撤销或中止底层存储工作。
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
