use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use super::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreferenceWorkDeadline {
    deadline: Option<Instant>,
}

impl PreferenceWorkDeadline {
    pub const fn none() -> Self {
        Self { deadline: None }
    }

    pub const fn at(deadline: Instant) -> Self {
        Self {
            deadline: Some(deadline),
        }
    }

    pub const fn instant(self) -> Option<Instant> {
        self.deadline
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceDurabilityState {
    Durable,
    Pending,
    VisibleNotDurable,
}

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

pub trait PreferenceMutationTicket: Send + Sync + fmt::Debug + 'static {
    fn generation(&self) -> u64;

    fn terminal(&self) -> Option<PreferenceMutationTerminal>;

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult;
}

pub trait PreferenceMutationCancellation: Send + Sync + fmt::Debug + 'static {
    fn cancel_before_start(&self) -> Result<(), PreferenceMutationCancelError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceMutationCancelError {
    AlreadyStarted,
    FencePinned,
    WrongAuthority,
}

pub struct PreferenceMutationSubmission {
    ticket: Arc<dyn PreferenceMutationTicket>,
    cancellation: Arc<dyn PreferenceMutationCancellation>,
}

impl PreferenceMutationSubmission {
    pub(crate) fn new(
        ticket: Arc<dyn PreferenceMutationTicket>,
        cancellation: Arc<dyn PreferenceMutationCancellation>,
    ) -> Self {
        Self {
            ticket,
            cancellation,
        }
    }

    pub fn ticket(&self) -> Arc<dyn PreferenceMutationTicket> {
        Arc::clone(&self.ticket)
    }

    pub fn cancellation(&self) -> Arc<dyn PreferenceMutationCancellation> {
        Arc::clone(&self.cancellation)
    }
}

impl fmt::Debug for PreferenceMutationSubmission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreferenceMutationSubmission")
            .field("generation", &self.ticket.generation())
            .field("terminal", &self.ticket.terminal())
            .finish_non_exhaustive()
    }
}

pub trait PreferenceFlushTicket: Send + Sync + fmt::Debug + 'static {
    fn epoch(&self) -> u64;

    fn terminal(&self) -> Option<PreferenceMutationTerminal>;

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceReadSnapshot {
    generation: u64,
    value: Option<Arc<[u8]>>,
    durability: PreferenceDurabilityState,
    last_terminal: Option<PreferenceMutationTerminal>,
}

impl PreferenceReadSnapshot {
    pub(crate) fn new(
        generation: u64,
        value: Option<Arc<[u8]>>,
        durability: PreferenceDurabilityState,
        last_terminal: Option<PreferenceMutationTerminal>,
    ) -> Self {
        Self {
            generation,
            value,
            durability,
            last_terminal,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.value.as_deref()
    }

    pub const fn durability(&self) -> PreferenceDurabilityState {
        self.durability
    }

    pub fn last_terminal(&self) -> Option<&PreferenceMutationTerminal> {
        self.last_terminal.as_ref()
    }
}

/// Versioned manager contract consumed by runtime clients and host adapters.
pub trait PreferenceStorage: Send + Sync + 'static {
    fn backend_kind(&self) -> PreferenceStorageBackendKind;

    fn snapshot(
        &self,
        key: &PreferenceKey,
    ) -> Result<PreferenceReadSnapshot, PreferenceStorageError>;

    fn submit_write(
        &self,
        key: PreferenceKey,
        value: Arc<[u8]>,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError>;

    fn submit_remove(
        &self,
        key: PreferenceKey,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError>;

    fn flush_fence(
        &self,
        deadline: PreferenceWorkDeadline,
    ) -> Result<Arc<dyn PreferenceFlushTicket>, PreferenceStorageError>;
}
