use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use crate::core::runtime::TaskGraphAdmissionError;

use super::super::{
    RenderArtifactBlockDescriptor, RenderArtifactIoPriority, RenderArtifactStoreLimits,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactBlockLoaderLimits {
    max_entries: usize,
    max_total_tickets: usize,
    max_tickets_per_entry: usize,
    max_retained_bytes: usize,
    max_decoded_block_bytes: u64,
    store_limits: RenderArtifactStoreLimits,
}

#[derive(Clone, Debug)]
pub struct RenderArtifactBlockRequest {
    descriptor: RenderArtifactBlockDescriptor,
    priority: RenderArtifactIoPriority,
    deadline: Option<std::time::Instant>,
}

impl RenderArtifactBlockRequest {
    pub fn new(
        descriptor: RenderArtifactBlockDescriptor,
        priority: RenderArtifactIoPriority,
    ) -> Self {
        Self {
            descriptor,
            priority,
            deadline: None,
        }
    }

    pub fn with_deadline(mut self, deadline: std::time::Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn descriptor(&self) -> &RenderArtifactBlockDescriptor {
        &self.descriptor
    }

    pub fn priority(&self) -> RenderArtifactIoPriority {
        self.priority
    }

    pub fn deadline(&self) -> Option<std::time::Instant> {
        self.deadline
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactBlockIoDispatchBudget {
    max_tasks: usize,
    max_encoded_bytes: u64,
}

impl RenderArtifactBlockIoDispatchBudget {
    pub const fn new(max_tasks: usize, max_encoded_bytes: u64) -> Self {
        Self {
            max_tasks,
            max_encoded_bytes,
        }
    }

    pub const fn max_tasks(self) -> usize {
        self.max_tasks
    }

    pub const fn max_encoded_bytes(self) -> u64 {
        self.max_encoded_bytes
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactBlockIoDispatchReport {
    pub submitted_tasks: usize,
    pub submitted_encoded_bytes: u64,
    pub remaining_queued_entries: usize,
    pub budget_exhausted: bool,
}

impl RenderArtifactBlockLoaderLimits {
    pub const fn new(
        max_entries: usize,
        max_total_tickets: usize,
        max_tickets_per_entry: usize,
        max_retained_bytes: usize,
        max_decoded_block_bytes: u64,
        store_limits: RenderArtifactStoreLimits,
    ) -> Self {
        Self {
            max_entries,
            max_total_tickets,
            max_tickets_per_entry,
            max_retained_bytes,
            max_decoded_block_bytes,
            store_limits,
        }
    }

    pub const fn max_entries(self) -> usize {
        self.max_entries
    }

    pub const fn max_total_tickets(self) -> usize {
        self.max_total_tickets
    }

    pub const fn max_tickets_per_entry(self) -> usize {
        self.max_tickets_per_entry
    }

    pub const fn max_retained_bytes(self) -> usize {
        self.max_retained_bytes
    }

    pub const fn max_decoded_block_bytes(self) -> u64 {
        self.max_decoded_block_bytes
    }

    pub const fn store_limits(self) -> RenderArtifactStoreLimits {
        self.store_limits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactBlockLoadStage {
    QueuedIo,
    Reading,
    QueuedDecode,
    Decoding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactBlockCancelReason {
    Caller,
    Deadline,
    OwnerClosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactBlockFailureCode {
    StoreUnavailable,
    StoreLimitExceeded,
    BlockSizeMismatch,
    BlockHashMismatch,
    DecodeFailed,
    DecodedSizeMismatch,
    DecodeAdmissionFailed,
}

#[derive(Clone, Debug)]
pub struct RenderArtifactBlockFailure {
    code: RenderArtifactBlockFailureCode,
    detail: Arc<str>,
}

impl RenderArtifactBlockFailure {
    pub(super) fn new(code: RenderArtifactBlockFailureCode, detail: impl Into<Arc<str>>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub const fn code(&self) -> RenderArtifactBlockFailureCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }
}

impl fmt::Display for RenderArtifactBlockFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.detail)
    }
}

impl std::error::Error for RenderArtifactBlockFailure {}

#[derive(Clone, Debug)]
pub struct RenderArtifactDecodedBlock {
    descriptor: RenderArtifactBlockDescriptor,
    bytes: Arc<[u8]>,
}

impl RenderArtifactDecodedBlock {
    pub(super) fn new(descriptor: RenderArtifactBlockDescriptor, bytes: Arc<[u8]>) -> Self {
        Self { descriptor, bytes }
    }

    pub const fn descriptor(&self) -> &RenderArtifactBlockDescriptor {
        &self.descriptor
    }

    pub const fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }
}

#[derive(Clone, Debug)]
pub enum RenderArtifactBlockPoll {
    Pending(RenderArtifactBlockLoadStage),
    Ready(RenderArtifactDecodedBlock),
    Failed(Arc<RenderArtifactBlockFailure>),
    Cancelled(RenderArtifactBlockCancelReason),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactBlockLoaderDiagnostics {
    pub live_entries: usize,
    pub live_tickets: usize,
    pub queued_io_entries: usize,
    pub retained_bytes: usize,
    pub submitted_io_tasks: u64,
    pub submitted_decode_tasks: u64,
    pub merged_requests: u64,
    pub ready_entries: u64,
    pub failed_entries: u64,
    pub cancelled_entries: u64,
    pub expired_tickets: u64,
    pub encoded_bytes_read: u64,
    pub decoded_bytes: u64,
    pub io_worker_wall: Duration,
    pub decode_worker_wall: Duration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactBlockMaintenanceReport {
    pub expired_tickets: usize,
    pub cancelled_entries: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactBlockLoaderCloseReport {
    pub cancelled_entries: usize,
    pub cancelled_tickets: usize,
    pub released_retained_bytes: usize,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactBlockLoaderInitError {
    #[error("render artifact block loader limit `{limit}` must be non-zero")]
    ZeroLimit { limit: &'static str },
    #[error("render artifact block loader scope capacity overflow")]
    ScopeCapacityOverflow,
    #[error(transparent)]
    Execution(#[from] TaskGraphAdmissionError),
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactBlockAdmissionError {
    #[error("render artifact block loader is closed")]
    Closed,
    #[error("render artifact block request batch must not be empty")]
    EmptyBatch,
    #[error("render artifact block entry capacity {capacity} is full")]
    EntryCapacityExceeded { capacity: usize },
    #[error("render artifact block ticket capacity {capacity} is full")]
    TicketCapacityExceeded { capacity: usize },
    #[error("render artifact block ticket capacity {capacity} is full for this block")]
    EntryTicketCapacityExceeded { capacity: usize },
    #[error("invalid render artifact block descriptor: {reason}")]
    InvalidBlockDescriptor { reason: &'static str },
    #[error("render artifact block decoded size {actual} exceeds limit {limit}")]
    DecodedBlockLimitExceeded { actual: u64, limit: u64 },
    #[error("render artifact block encoded size {actual} exceeds limit {limit}")]
    EncodedBlockLimitExceeded { actual: u64, limit: u64 },
    #[error("render artifact block retained-byte quote overflow")]
    RetainedBytesOverflow,
    #[error(
        "render artifact block retained-byte request {requested} exceeds remaining capacity {remaining}"
    )]
    RetainedBytesCapacityExceeded { requested: usize, remaining: usize },
    #[error("render artifact block I/O frontier identifier space is exhausted")]
    FrontierSequenceExhausted,
    #[error("render artifact block ticket identifier space is exhausted")]
    TicketIdExhausted,
    #[error("render artifact block admission invariant failed: {reason}")]
    InternalInvariant { reason: &'static str },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactBlockIoDispatchError {
    #[error("render artifact block loader is closed")]
    Closed,
    #[error("render artifact block I/O dispatch limit `{limit}` must be non-zero")]
    ZeroLimit { limit: &'static str },
    #[error("render artifact block task identifier space is exhausted")]
    TaskIdExhausted,
    #[error(transparent)]
    Execution(#[from] TaskGraphAdmissionError),
}
