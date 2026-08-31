use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use crate::core::resource::UntypedResourceHandle;
use crate::core::runtime::TaskGraphAdmissionError;

use super::super::RenderArtifactIoPriority;
use super::super::{RenderArtifactManifest, RenderArtifactStoreLimits};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderArtifactManifestRequestKey {
    resource: UntypedResourceHandle,
    asset_revision: u64,
    target_platform: Arc<str>,
}

impl RenderArtifactManifestRequestKey {
    pub fn new(
        resource: UntypedResourceHandle,
        asset_revision: u64,
        target_platform: Arc<str>,
    ) -> Self {
        Self {
            resource,
            asset_revision,
            target_platform,
        }
    }

    pub const fn resource(&self) -> UntypedResourceHandle {
        self.resource
    }

    pub const fn asset_revision(&self) -> u64 {
        self.asset_revision
    }

    pub fn target_platform(&self) -> &str {
        self.target_platform.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct RenderArtifactManifestRequest {
    key: RenderArtifactManifestRequestKey,
    priority: RenderArtifactIoPriority,
    deadline: Option<std::time::Instant>,
}

impl RenderArtifactManifestRequest {
    pub fn new(
        resource: UntypedResourceHandle,
        asset_revision: u64,
        target_platform: Arc<str>,
        priority: RenderArtifactIoPriority,
    ) -> Self {
        Self {
            key: RenderArtifactManifestRequestKey::new(resource, asset_revision, target_platform),
            priority,
            deadline: None,
        }
    }

    pub fn with_deadline(mut self, deadline: std::time::Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub const fn key(&self) -> &RenderArtifactManifestRequestKey {
        &self.key
    }

    pub const fn priority(&self) -> RenderArtifactIoPriority {
        self.priority
    }

    pub const fn deadline(&self) -> Option<std::time::Instant> {
        self.deadline
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactManifestIoDispatchBudget {
    max_tasks: usize,
}

impl RenderArtifactManifestIoDispatchBudget {
    pub const fn new(max_tasks: usize) -> Self {
        Self { max_tasks }
    }

    pub const fn max_tasks(self) -> usize {
        self.max_tasks
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactManifestIoDispatchReport {
    pub submitted_tasks: usize,
    pub remaining_queued_entries: usize,
    pub budget_exhausted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactManifestLoaderLimits {
    max_entries: usize,
    max_total_tickets: usize,
    max_tickets_per_entry: usize,
    max_retained_bytes: usize,
    store_limits: RenderArtifactStoreLimits,
}

impl RenderArtifactManifestLoaderLimits {
    pub const fn new(
        max_entries: usize,
        max_total_tickets: usize,
        max_tickets_per_entry: usize,
        max_retained_bytes: usize,
        store_limits: RenderArtifactStoreLimits,
    ) -> Self {
        Self {
            max_entries,
            max_total_tickets,
            max_tickets_per_entry,
            max_retained_bytes,
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

    pub const fn store_limits(self) -> RenderArtifactStoreLimits {
        self.store_limits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactManifestLoadStage {
    QueuedIo,
    Reading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactManifestCancelReason {
    Caller,
    Deadline,
    OwnerClosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactManifestFailureCode {
    NotFound,
    StoreLimitExceeded,
    InvalidManifest,
    StoreUnavailable,
}

#[derive(Clone, Debug)]
pub struct RenderArtifactManifestFailure {
    code: RenderArtifactManifestFailureCode,
    detail: Arc<str>,
}

impl RenderArtifactManifestFailure {
    pub(super) fn new(
        code: RenderArtifactManifestFailureCode,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub const fn code(&self) -> RenderArtifactManifestFailureCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }
}

impl fmt::Display for RenderArtifactManifestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.detail)
    }
}

impl std::error::Error for RenderArtifactManifestFailure {}

#[derive(Clone, Debug)]
pub enum RenderArtifactManifestPoll {
    Pending(RenderArtifactManifestLoadStage),
    Ready(Arc<RenderArtifactManifest>),
    Failed(Arc<RenderArtifactManifestFailure>),
    Cancelled(RenderArtifactManifestCancelReason),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactManifestLoaderDiagnostics {
    pub live_entries: usize,
    pub live_tickets: usize,
    pub queued_io_entries: usize,
    pub reserved_retained_bytes: usize,
    pub submitted_io_tasks: u64,
    pub merged_requests: u64,
    pub ready_entries: u64,
    pub failed_entries: u64,
    pub cancelled_entries: u64,
    pub expired_tickets: u64,
    pub io_worker_wall: Duration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactManifestMaintenanceReport {
    pub expired_tickets: usize,
    pub cancelled_entries: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderArtifactManifestLoaderCloseReport {
    pub cancelled_entries: usize,
    pub cancelled_tickets: usize,
    pub released_reserved_bytes: usize,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactManifestLoaderInitError {
    #[error("render artifact manifest loader limit `{limit}` must be non-zero")]
    ZeroLimit { limit: &'static str },
    #[error("render artifact manifest loader retained-byte quote overflowed")]
    RetainedBytesQuoteOverflow,
    #[error(
        "render artifact manifest loader requires {required} retained bytes per entry but capacity is {capacity}"
    )]
    RetainedBytesCapacityTooSmall { required: usize, capacity: usize },
    #[error(transparent)]
    Execution(#[from] TaskGraphAdmissionError),
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactManifestAdmissionError {
    #[error("render artifact manifest loader is closed")]
    Closed,
    #[error("render artifact manifest request batch must not be empty")]
    EmptyBatch,
    #[error("render artifact manifest target platform must not be empty")]
    EmptyTargetPlatform,
    #[error("render artifact manifest entry capacity {capacity} is full")]
    EntryCapacityExceeded { capacity: usize },
    #[error("render artifact manifest ticket capacity {capacity} is full")]
    TicketCapacityExceeded { capacity: usize },
    #[error("render artifact manifest ticket capacity {capacity} is full for this identity")]
    EntryTicketCapacityExceeded { capacity: usize },
    #[error(
        "render artifact manifest retained-byte request {requested} exceeds remaining capacity {remaining}"
    )]
    RetainedBytesCapacityExceeded { requested: usize, remaining: usize },
    #[error("render artifact manifest retained-byte request overflowed")]
    RetainedBytesRequestOverflow,
    #[error("render artifact manifest I/O frontier identifier space is exhausted")]
    FrontierSequenceExhausted,
    #[error("render artifact manifest ticket identifier space is exhausted")]
    TicketIdExhausted,
    #[error("render artifact manifest admission invariant failed: {reason}")]
    InternalInvariant { reason: &'static str },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactManifestIoDispatchError {
    #[error("render artifact manifest loader is closed")]
    Closed,
    #[error("render artifact manifest I/O dispatch max_tasks must be non-zero")]
    ZeroTaskLimit,
    #[error("render artifact manifest task identifier space is exhausted")]
    TaskIdExhausted,
    #[error(transparent)]
    Execution(#[from] TaskGraphAdmissionError),
}
