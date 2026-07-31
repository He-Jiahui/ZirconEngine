use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Instant;

use crate::core::framework::platform::{
    PreferenceDurabilityState, PreferenceFlushTicket, PreferenceKey, PreferenceMutationCancelError,
    PreferenceMutationCancellation, PreferenceMutationSubmission, PreferenceMutationTerminal,
    PreferenceMutationTicket, PreferencePersistenceFailureProjection, PreferenceReadSnapshot,
    PreferenceStorageBackendKind, PreferenceStorageError, PreferenceStorageErrorKind,
    PreferenceStorageOperation, PreferenceTicketWaitResult, PreferenceWorkDeadline,
};
use crate::core::runtime::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority, BoundedKeyedIoCancelError,
    BoundedKeyedIoFence, BoundedKeyedIoLane, BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard,
    BoundedKeyedIoTicket, BoundedKeyedIoWaitResult, BoundedKeyedIoWorkDeadline,
    GlobalAdmissionEpoch, JobScheduler, TaskPools,
};
use crate::platform::preferences::PreferenceStorageBackend;

use super::overlay::{map_lane_terminal, PreferenceOverlay, PreferenceOverlayLimits};
use super::work::{lane_failure, perform_flush, perform_read, perform_remove, perform_write};

pub const MAX_PREFERENCE_VALUE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_PREFERENCE_FAILURE_DETAIL_BYTES: usize = 1024;
const OVERLAY_ENTRY_METADATA_BYTES: usize = 128;
const LANE_ENTRY_METADATA_BYTES: usize = 128;
const VISIBLE_NOT_DURABLE_STATE: &str = "visible_not_durable";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreferencePersistenceLimits {
    pub max_value_bytes: usize,
    pub max_overlay_entries: usize,
    pub max_overlay_retained_bytes: usize,
    pub max_lane_entries: usize,
    pub max_lane_retained_bytes: usize,
}

impl Default for PreferencePersistenceLimits {
    fn default() -> Self {
        Self {
            max_value_bytes: MAX_PREFERENCE_VALUE_BYTES,
            max_overlay_entries: 4096,
            max_overlay_retained_bytes: 128 * 1024 * 1024,
            max_lane_entries: 4096,
            max_lane_retained_bytes: 128 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreferencePersistenceQuote {
    pub overlay_retained_bytes: usize,
    pub lane_retained_bytes: usize,
}

impl PreferencePersistenceQuote {
    fn quote_retained_bytes(key: &PreferenceKey, value_bytes: usize) -> Option<Self> {
        let key_bytes = key.namespace().len().checked_add(key.key().len())?;
        let bounded_projection = MAX_PREFERENCE_FAILURE_DETAIL_BYTES;
        Some(Self {
            overlay_retained_bytes: key_bytes
                .checked_add(value_bytes)?
                .checked_add(OVERLAY_ENTRY_METADATA_BYTES)?
                .checked_add(bounded_projection)?,
            lane_retained_bytes: key_bytes
                .checked_add(value_bytes)?
                .checked_add(LANE_ENTRY_METADATA_BYTES)?
                .checked_add(bounded_projection)?,
        })
    }

    fn for_fence() -> Option<Self> {
        Some(Self {
            overlay_retained_bytes: 0,
            lane_retained_bytes: LANE_ENTRY_METADATA_BYTES
                .checked_add(MAX_PREFERENCE_FAILURE_DETAIL_BYTES)?,
        })
    }
}

pub struct PreferencePersistenceAdapter {
    backend: RwLock<Arc<dyn PreferenceStorageBackend>>,
    lane: BoundedKeyedIoLane,
    overlay: PreferenceOverlay,
    limits: PreferencePersistenceLimits,
}

impl PreferencePersistenceAdapter {
    pub fn new(
        backend: Arc<dyn PreferenceStorageBackend>,
        limits: PreferencePersistenceLimits,
    ) -> Self {
        let scheduler = JobScheduler::from_pool(TaskPools::process_default().io().clone());
        Self {
            backend: RwLock::new(backend),
            lane: BoundedKeyedIoLane::new(
                BoundedKeyedIoLimits::new(limits.max_lane_entries, limits.max_lane_retained_bytes),
                scheduler,
            ),
            overlay: PreferenceOverlay::new(PreferenceOverlayLimits {
                max_entries: limits.max_overlay_entries,
                max_retained_bytes: limits.max_overlay_retained_bytes,
            }),
            limits,
        }
    }

    pub fn backend_kind(&self) -> PreferenceStorageBackendKind {
        self.backend().backend_kind()
    }

    pub(crate) fn replace_backend(&self, backend: Arc<dyn PreferenceStorageBackend>) {
        *self.backend_mut() = backend;
    }

    pub fn snapshot(
        &self,
        key: &PreferenceKey,
    ) -> Result<PreferenceReadSnapshot, PreferenceStorageError> {
        if let Some(snapshot) = self.snapshot_if_present(key) {
            return Ok(snapshot);
        }
        self.submit_initial_read(key.clone())?;
        self.snapshot_if_present(key).ok_or_else(|| {
            immediate_error(
                PreferenceStorageErrorKind::TransientIo,
                PreferenceStorageOperation::Read,
                "overlay generation disappeared after read admission",
            )
        })
    }

    pub fn submit_write(
        &self,
        key: PreferenceKey,
        value: Arc<[u8]>,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError> {
        if value.len() > self.limits.max_value_bytes {
            return Err(immediate_error(
                PreferenceStorageErrorKind::CapacityExceeded,
                PreferenceStorageOperation::Write,
                "preference value exceeds configured maximum",
            ));
        }
        self.submit_mutation(
            key,
            Some(value),
            deadline,
            PreferenceStorageOperation::Write,
        )
    }

    pub fn submit_remove(
        &self,
        key: PreferenceKey,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError> {
        self.submit_mutation(key, None, deadline, PreferenceStorageOperation::Remove)
    }

    pub fn flush_fence(
        &self,
        deadline: PreferenceWorkDeadline,
    ) -> Result<Arc<dyn PreferenceFlushTicket>, PreferenceStorageError> {
        let quote = PreferencePersistenceQuote::for_fence().ok_or_else(|| {
            immediate_error(
                PreferenceStorageErrorKind::CapacityExceeded,
                PreferenceStorageOperation::Flush,
                "preference fence quote overflow",
            )
        })?;
        let backend = self.backend();
        let projection = Arc::new(Mutex::new(None));
        let projection_for_work = Arc::clone(&projection);
        let fence = self
            .lane
            .submit_fence(
                quote.lane_retained_bytes,
                lane_deadline(deadline),
                Box::new(move || match perform_flush(&backend) {
                    Ok(()) => Ok(()),
                    Err(failure) => {
                        *lock(&projection_for_work) = Some(failure.clone());
                        Err(lane_failure(&failure))
                    }
                }),
            )
            .map_err(|error| admission_error(error, PreferenceStorageOperation::Flush))?;
        Ok(Arc::new(PreferenceFenceView { fence, projection }))
    }

    pub fn diagnostics(&self) -> crate::core::runtime::BoundedKeyedIoDiagnostics {
        self.lane.diagnostics()
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        self.lane.shutdown()
    }

    fn submit_initial_read(&self, key: PreferenceKey) -> Result<(), PreferenceStorageError> {
        let quote =
            PreferencePersistenceQuote::quote_retained_bytes(&key, self.limits.max_value_bytes)
                .ok_or_else(|| {
                    immediate_error(
                        PreferenceStorageErrorKind::CapacityExceeded,
                        PreferenceStorageOperation::Read,
                        "preference read quote overflow",
                    )
                })?;
        let reservation = self.overlay.reserve(
            quote.overlay_retained_bytes,
            PreferenceStorageOperation::Read,
        )?;
        let generation = reservation.generation();
        let backend = self.backend();
        let overlay = self.overlay.clone();
        let key_for_work = key.clone();
        let max_value_bytes = self.limits.max_value_bytes;
        let admission = self
            .lane
            .try_admit(
                opaque_key(&key),
                generation,
                quote.lane_retained_bytes,
                BoundedKeyedIoWorkDeadline::none(),
                Box::new(move || {
                    let result = perform_read(&backend, &key_for_work, max_value_bytes);
                    overlay.complete_read(&key_for_work, generation, result.clone());
                    result.map(|_| ()).map_err(|failure| lane_failure(&failure))
                }),
            )
            .map_err(|error| admission_error(error, PreferenceStorageOperation::Read))?;
        reservation.install_generation_before_runnable(
            key,
            None,
            PreferenceDurabilityState::Pending,
        );
        admission.activate();
        Ok(())
    }

    fn submit_mutation(
        &self,
        key: PreferenceKey,
        value: Option<Arc<[u8]>>,
        deadline: PreferenceWorkDeadline,
        operation: PreferenceStorageOperation,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError> {
        let value_bytes = value.as_ref().map_or(0, |value| value.len());
        let quote = PreferencePersistenceQuote::quote_retained_bytes(&key, value_bytes)
            .ok_or_else(|| {
                immediate_error(
                    PreferenceStorageErrorKind::CapacityExceeded,
                    operation,
                    "preference mutation quote overflow",
                )
            })?;
        let reservation = self
            .overlay
            .reserve(quote.overlay_retained_bytes, operation)?;
        let generation = reservation.generation();
        let backend = self.backend();
        let overlay = self.overlay.clone();
        let key_for_work = key.clone();
        let value_for_work = value.clone();
        let admission = self
            .lane
            .try_admit(
                opaque_key(&key),
                generation,
                quote.lane_retained_bytes,
                lane_deadline(deadline),
                Box::new(move || {
                    let result = match &value_for_work {
                        Some(value) => perform_write(&backend, &key_for_work, value),
                        None => perform_remove(&backend, &key_for_work),
                    };
                    overlay.complete_mutation(&key_for_work, generation, result.clone());
                    result.map_err(|failure| lane_failure(&failure))
                }),
            )
            .map_err(|error| admission_error(error, operation))?;
        let lane_ticket = admission.ticket();
        let cancel_authority = admission.cancel_authority();
        reservation.install_generation_before_runnable(
            key.clone(),
            value,
            PreferenceDurabilityState::Pending,
        );
        admission.activate();
        let view = Arc::new(PreferenceTicketView {
            ticket: lane_ticket.clone(),
            overlay: self.overlay.clone(),
            key,
            generation,
        });
        let cancellation = Arc::new(PreferenceCancellationView {
            ticket: lane_ticket,
            authority: cancel_authority,
            view: Arc::clone(&view),
        });
        Ok(PreferenceMutationSubmission::new(view, cancellation))
    }

    fn snapshot_if_present(&self, key: &PreferenceKey) -> Option<PreferenceReadSnapshot> {
        self.overlay.snapshot(key)
    }

    fn backend(&self) -> Arc<dyn PreferenceStorageBackend> {
        Arc::clone(&self.backend_read())
    }

    fn backend_read(&self) -> RwLockReadGuard<'_, Arc<dyn PreferenceStorageBackend>> {
        self.backend
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn backend_mut(&self) -> RwLockWriteGuard<'_, Arc<dyn PreferenceStorageBackend>> {
        self.backend
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for PreferencePersistenceAdapter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreferencePersistenceAdapter")
            .field("backend_kind", &self.backend_kind())
            .field("diagnostics", &self.diagnostics())
            .field("failure_state", &VISIBLE_NOT_DURABLE_STATE)
            .finish_non_exhaustive()
    }
}

impl Drop for PreferencePersistenceAdapter {
    fn drop(&mut self) {
        drop(self.lane.shutdown());
    }
}

#[derive(Clone)]
struct PreferenceTicketView {
    ticket: BoundedKeyedIoTicket,
    overlay: PreferenceOverlay,
    key: PreferenceKey,
    generation: u64,
}

impl PreferenceTicketView {
    fn projected_terminal(&self) -> Option<PreferenceMutationTerminal> {
        let lane_terminal = self.ticket.terminal()?;
        self.overlay
            .reflect_lane_terminal(&self.key, self.generation, lane_terminal);
        self.overlay
            .terminal_for(&self.key, self.generation)
            .or_else(|| map_lane_terminal(lane_terminal, None))
    }
}

impl fmt::Debug for PreferenceTicketView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreferenceMutationTicket")
            .field("generation", &self.generation)
            .field("terminal", &self.projected_terminal())
            .finish()
    }
}

impl PreferenceMutationTicket for PreferenceTicketView {
    fn generation(&self) -> u64 {
        self.generation
    }

    fn terminal(&self) -> Option<PreferenceMutationTerminal> {
        self.projected_terminal()
    }

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult {
        match self.ticket.wait_until(deadline) {
            BoundedKeyedIoWaitResult::ObserverTimedOut => {
                PreferenceTicketWaitResult::ObserverTimedOut
            }
            BoundedKeyedIoWaitResult::Terminal(terminal) => {
                self.overlay
                    .reflect_lane_terminal(&self.key, self.generation, terminal);
                PreferenceTicketWaitResult::Terminal(
                    self.projected_terminal()
                        .unwrap_or(PreferenceMutationTerminal::Shutdown),
                )
            }
        }
    }
}

struct PreferenceCancellationView {
    ticket: BoundedKeyedIoTicket,
    authority: BoundedKeyedIoCancelAuthority,
    view: Arc<PreferenceTicketView>,
}

impl fmt::Debug for PreferenceCancellationView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreferenceMutationCancellation")
            .field("generation", &self.view.generation)
            .finish_non_exhaustive()
    }
}

impl PreferenceMutationCancellation for PreferenceCancellationView {
    fn cancel_before_start(&self) -> Result<(), PreferenceMutationCancelError> {
        self.ticket
            .cancel_before_start(&self.authority)
            .map_err(|error| match error {
                BoundedKeyedIoCancelError::WrongAuthority => {
                    PreferenceMutationCancelError::WrongAuthority
                }
                BoundedKeyedIoCancelError::AlreadyStarted => {
                    PreferenceMutationCancelError::AlreadyStarted
                }
                BoundedKeyedIoCancelError::FencePinned => {
                    PreferenceMutationCancelError::FencePinned
                }
            })?;
        let _ = self.view.projected_terminal();
        Ok(())
    }
}

struct PreferenceFenceView {
    fence: BoundedKeyedIoFence,
    projection: Arc<Mutex<Option<PreferencePersistenceFailureProjection>>>,
}

impl fmt::Debug for PreferenceFenceView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreferenceFlushTicket")
            .field("epoch", &self.epoch())
            .field("terminal", &self.terminal())
            .finish()
    }
}

impl PreferenceFlushTicket for PreferenceFenceView {
    fn epoch(&self) -> u64 {
        let epoch: GlobalAdmissionEpoch = self.fence.epoch();
        epoch.value()
    }

    fn terminal(&self) -> Option<PreferenceMutationTerminal> {
        let lane = self.fence.ticket().terminal()?;
        map_fence_terminal(lane, lock(&self.projection).clone())
    }

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult {
        match self.fence.ticket().wait_until(deadline) {
            BoundedKeyedIoWaitResult::ObserverTimedOut => {
                PreferenceTicketWaitResult::ObserverTimedOut
            }
            BoundedKeyedIoWaitResult::Terminal(terminal) => PreferenceTicketWaitResult::Terminal(
                map_fence_terminal(terminal, lock(&self.projection).clone())
                    .unwrap_or(PreferenceMutationTerminal::Shutdown),
            ),
        }
    }
}

fn map_fence_terminal(
    terminal: crate::core::runtime::BoundedKeyedIoTerminal,
    projection: Option<PreferencePersistenceFailureProjection>,
) -> Option<PreferenceMutationTerminal> {
    let projection = projection.or_else(|| match terminal {
        crate::core::runtime::BoundedKeyedIoTerminal::Failed(failure) => {
            Some(PreferencePersistenceFailureProjection::new(
                PreferenceStorageErrorKind::TransientIo,
                PreferenceStorageOperation::Flush,
                "persistence_lane",
                failure.code.to_owned(),
            ))
        }
        _ => None,
    });
    map_lane_terminal(terminal, projection)
}

fn opaque_key(key: &PreferenceKey) -> Arc<str> {
    Arc::from(format!("{}\0{}", key.namespace(), key.key()))
}

fn lane_deadline(deadline: PreferenceWorkDeadline) -> BoundedKeyedIoWorkDeadline {
    deadline.instant().map_or_else(
        BoundedKeyedIoWorkDeadline::none,
        BoundedKeyedIoWorkDeadline::at,
    )
}

fn admission_error(
    error: BoundedKeyedIoAdmissionError,
    operation: PreferenceStorageOperation,
) -> PreferenceStorageError {
    let (kind, detail) = match error {
        BoundedKeyedIoAdmissionError::Closed => (
            PreferenceStorageErrorKind::Unavailable,
            "preference persistence lane is closed",
        ),
        BoundedKeyedIoAdmissionError::EntryCapacityExceeded => (
            PreferenceStorageErrorKind::CapacityExceeded,
            "preference persistence lane entry capacity exceeded",
        ),
        BoundedKeyedIoAdmissionError::RetainedBytesCapacityExceeded => (
            PreferenceStorageErrorKind::CapacityExceeded,
            "preference persistence lane retained-byte capacity exceeded",
        ),
        BoundedKeyedIoAdmissionError::RetainedBytesOverflow => (
            PreferenceStorageErrorKind::CapacityExceeded,
            "preference persistence lane retained-byte quote overflow",
        ),
    };
    PreferenceStorageError::new(kind, operation, "persistence_lane", detail)
}

fn immediate_error(
    kind: PreferenceStorageErrorKind,
    operation: PreferenceStorageOperation,
    detail: &'static str,
) -> PreferenceStorageError {
    PreferenceStorageError::new(kind, operation, "persistence_adapter", detail)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
