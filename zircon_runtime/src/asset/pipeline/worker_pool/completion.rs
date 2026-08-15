//! Completion publication, retention, and expiry for the asset IO worker pool.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::asset::types::{AssetRequest, CpuAssetPayload};
use crate::core::runtime::tasks::{TaskTimer, TaskTimerSubscription};
use crate::core::CoreResult;

use super::diagnostics::record_duration_measurement;
use super::payload::payload_bytes;
use super::{AssetWorkerPoolDiagnostics, AssetWorkerPoolOptions};

pub(super) struct CompletionRegistry {
    pub(super) state: Mutex<CompletionRegistryState>,
}

impl CompletionRegistry {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(CompletionRegistryState {
                closing: false,
                scheduled_jobs: 0,
                in_flight: HashMap::new(),
                completed: HashMap::new(),
                completed_bytes: 0,
            }),
        }
    }
}

pub(super) struct CompletionRegistryState {
    pub(super) closing: bool,
    // Admission remains charged until the queued task closure naturally exits.
    pub(super) scheduled_jobs: usize,
    pub(super) in_flight: HashMap<AssetRequest, Arc<CompletionEntry>>,
    pub(super) completed: HashMap<AssetRequest, CompletedEntry>,
    pub(super) completed_bytes: usize,
}

pub(super) struct CompletedEntry {
    pub(super) entry: Arc<CompletionEntry>,
    pub(super) bytes: usize,
}

pub(super) struct CompletionEntry {
    pub(super) request: AssetRequest,
    enqueued_at: Instant,
    pub(super) request_deadline: Instant,
    state: Mutex<CompletionEntryState>,
    expiry_subscription: Mutex<Option<TaskTimerSubscription>>,
    expiry_generation: AtomicU64,
    pub(super) changed: Condvar,
}

struct CompletionEntryState {
    waiters: usize,
    running: bool,
    terminal: CompletionTerminal,
}

#[derive(Clone)]
pub(super) enum CompletionTerminal {
    Pending,
    Ready {
        payload: Arc<CpuAssetPayload>,
        expires_at: Instant,
    },
    Cancelled,
    Expired,
    Rejected,
}

/// Observer handle for a single-flight asset request.
pub struct AssetWorkerCompletionTicket {
    pub(super) entry: Arc<CompletionEntry>,
    completions: Arc<CompletionRegistry>,
    pub(super) diagnostics: Arc<Mutex<AssetWorkerPoolDiagnostics>>,
}

impl AssetWorkerCompletionTicket {
    pub(super) fn new(
        entry: Arc<CompletionEntry>,
        completions: Arc<CompletionRegistry>,
        diagnostics: Arc<Mutex<AssetWorkerPoolDiagnostics>>,
    ) -> Self {
        Self {
            entry,
            completions,
            diagnostics,
        }
    }

    pub fn request(&self) -> &AssetRequest {
        &self.entry.request
    }

    pub fn try_result(&self) -> Result<Option<Arc<CpuAssetPayload>>, AssetWorkerCompletionError> {
        let mut state = lock_completion_entry(&self.entry);
        let now = Instant::now();
        let expired = expire_ticket_terminal(
            &mut state,
            self.entry.request_deadline,
            now,
            &self.entry.changed,
        );
        let result = if expired {
            Err(AssetWorkerCompletionError::Expired)
        } else {
            terminal_result(&state.terminal, now)
        };
        drop(state);
        if matches!(&result, Err(AssetWorkerCompletionError::Expired)) {
            unregister_expired_ticket(&self.completions, &self.diagnostics, &self.entry);
        }
        result
    }

    pub fn wait_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Arc<CpuAssetPayload>, AssetWorkerCompletionError> {
        let caller_deadline = deadline_after(Instant::now(), timeout);
        let deadline = caller_deadline.min(self.entry.request_deadline);
        let mut state = lock_completion_entry(&self.entry);
        loop {
            let now = Instant::now();
            let expired = expire_ticket_terminal(
                &mut state,
                self.entry.request_deadline,
                now,
                &self.entry.changed,
            );
            if expired {
                drop(state);
                unregister_expired_ticket(&self.completions, &self.diagnostics, &self.entry);
                return Err(AssetWorkerCompletionError::Expired);
            }
            match terminal_result(&state.terminal, now)? {
                Some(payload) => return Ok(payload),
                None => {}
            }
            if now >= caller_deadline {
                return Err(AssetWorkerCompletionError::TimedOut);
            }
            let remaining = deadline.saturating_duration_since(now);
            let (next_state, timed_out) = self
                .entry
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if timed_out.timed_out() {
                continue;
            }
        }
    }
}

impl fmt::Debug for AssetWorkerCompletionTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssetWorkerCompletionTicket")
            .field("request", self.request())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AssetWorkerCompletionError {
    #[error("asset worker completion did not arrive before the caller deadline")]
    TimedOut,
    #[error("asset worker completion was cancelled")]
    Cancelled,
    #[error("asset worker completion expired before it was harvested")]
    Expired,
    #[error("asset worker completion exceeded the shared completion budget")]
    Rejected,
}

/// Selects the lifecycle timer that owns asset-request expiry registrations.
#[derive(Clone)]
pub(super) enum AssetWorkerExpiryTimer {
    ProcessDefault,
    Explicit(TaskTimer),
}

impl AssetWorkerExpiryTimer {
    pub(super) fn schedule_at(
        &self,
        deadline: Instant,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> CoreResult<TaskTimerSubscription> {
        match self {
            Self::ProcessDefault => TaskTimer::process_default()?.schedule_at(deadline, callback),
            Self::Explicit(timer) => timer.schedule_at(deadline, callback),
        }
    }
}

pub(super) fn record_queue_age(
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
    queue_age: Duration,
) {
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    (
        diagnostics.queue_age_total,
        diagnostics.queue_age_max,
        diagnostics.queue_age_samples,
    ) = record_duration_measurement(
        diagnostics.queue_age_total,
        diagnostics.queue_age_max,
        diagnostics.queue_age_samples,
        queue_age,
    );
}

pub(super) fn publish_completion(
    expiry_timer: &AssetWorkerExpiryTimer,
    completions: &Arc<CompletionRegistry>,
    diagnostics: &Arc<Mutex<AssetWorkerPoolDiagnostics>>,
    options: &AssetWorkerPoolOptions,
    entry: Arc<CompletionEntry>,
    payload: Option<CpuAssetPayload>,
) {
    let now = Instant::now();
    let mut state = lock_completion_registry(completions);
    let expiry = expire_entries(&mut state, now);
    record_expiry_for_diagnostics(diagnostics, expiry);
    state.scheduled_jobs = state.scheduled_jobs.saturating_sub(1);
    let Some(current) = state.in_flight.get(&entry.request) else {
        return;
    };
    if !Arc::ptr_eq(current, &entry) {
        return;
    }
    let waiters = entry.waiter_count();
    let mut outcome = CompletionOutcome::Cancelled;
    if state.closing {
        state.in_flight.remove(&entry.request);
        entry.terminate(CompletionTerminal::Cancelled);
    } else if entry.is_request_expired(now) || payload.is_none() {
        state.in_flight.remove(&entry.request);
        entry.terminate(CompletionTerminal::Expired);
        outcome = CompletionOutcome::Expired;
    } else if let Some(payload) = payload {
        let bytes = payload_bytes(&payload);
        let exceeds_budget = bytes > options.completion_byte_capacity
            || state.completed.len() >= options.completion_entry_capacity
            || state.completed_bytes.saturating_add(bytes) > options.completion_byte_capacity;
        if exceeds_budget {
            state.in_flight.remove(&entry.request);
            entry.terminate(CompletionTerminal::Rejected);
            outcome = CompletionOutcome::Rejected;
        } else {
            let is_failure = matches!(payload, CpuAssetPayload::Failure { .. });
            let payload = Arc::new(payload);
            // One entry owns one timer slot; release the request deadline before arming completion retention.
            entry.cancel_expiry();
            let expires_at = deadline_after(Instant::now(), options.completion_max_age);
            let expiry_generation = entry.begin_expiry();
            match schedule_entry_expiry(
                expiry_timer,
                completions,
                diagnostics,
                &entry,
                expires_at,
                expiry_generation,
            ) {
                Ok(subscription) => {
                    entry.install_expiry_subscription(expiry_generation, subscription);
                    match entry.ready(payload, expires_at) {
                        ReadyTransition::Ready => {
                            state.in_flight.remove(&entry.request);
                            state.completed_bytes = state.completed_bytes.saturating_add(bytes);
                            state.completed.insert(
                                entry.request.clone(),
                                CompletedEntry {
                                    entry: Arc::clone(&entry),
                                    bytes,
                                },
                            );
                            outcome = CompletionOutcome::Ready { is_failure, bytes };
                        }
                        ReadyTransition::Expired | ReadyTransition::Terminal => {
                            state.in_flight.remove(&entry.request);
                            entry.cancel_expiry();
                            outcome = CompletionOutcome::Expired;
                        }
                    }
                }
                Err(_) => {
                    state.in_flight.remove(&entry.request);
                    entry.terminate(CompletionTerminal::Rejected);
                    outcome = CompletionOutcome::Rejected;
                }
            }
        }
    }
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.in_flight = diagnostics.in_flight.saturating_sub(1);
    diagnostics.in_flight_waiters = diagnostics.in_flight_waiters.saturating_sub(waiters);
    match outcome {
        CompletionOutcome::Ready { is_failure, bytes } => {
            diagnostics.completed = diagnostics.completed.saturating_add(1);
            diagnostics.failed = diagnostics.failed.saturating_add(u64::from(is_failure));
            diagnostics.completion_entries = diagnostics.completion_entries.saturating_add(1);
            diagnostics.completion_bytes = diagnostics.completion_bytes.saturating_add(bytes);
        }
        CompletionOutcome::Rejected => {
            diagnostics.rejected = diagnostics.rejected.saturating_add(1);
            diagnostics.completion_rejected = diagnostics.completion_rejected.saturating_add(1);
        }
        CompletionOutcome::Expired => {
            diagnostics.expired = diagnostics.expired.saturating_add(1);
        }
        CompletionOutcome::Cancelled => {
            diagnostics.cancelled = diagnostics.cancelled.saturating_add(1);
        }
    }
}

enum CompletionOutcome {
    Ready { is_failure: bool, bytes: usize },
    Rejected,
    Expired,
    Cancelled,
}

pub(super) fn expire_entries(state: &mut CompletionRegistryState, now: Instant) -> ExpiryReport {
    let expired_in_flight = state
        .in_flight
        .iter()
        .filter_map(|(request, entry)| entry.is_request_expired(now).then(|| request.clone()))
        .collect::<Vec<_>>();
    let expired_completed = state
        .completed
        .iter()
        .filter_map(|(request, completed)| {
            completed
                .entry
                .is_completion_expired(now)
                .then(|| request.clone())
        })
        .collect::<Vec<_>>();
    let mut report = ExpiryReport::default();
    for request in expired_in_flight {
        if let Some(entry) = state.in_flight.remove(&request) {
            report.in_flight_entries = report.in_flight_entries.saturating_add(1);
            let waiters = entry.waiter_count();
            report.in_flight_waiters = report.in_flight_waiters.saturating_add(waiters);
            entry.terminate(CompletionTerminal::Expired);
        }
    }
    for request in expired_completed {
        if let Some(completed) = state.completed.remove(&request) {
            state.completed_bytes = state.completed_bytes.saturating_sub(completed.bytes);
            report.completed_entries = report.completed_entries.saturating_add(1);
            report.completed_bytes = report.completed_bytes.saturating_add(completed.bytes);
            completed.entry.terminate(CompletionTerminal::Expired);
        }
    }
    report
}

pub(super) fn record_expiry_for_diagnostics(
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
    expiry: ExpiryReport,
) {
    if expiry.is_empty() {
        return;
    }
    let mut diagnostics = lock_worker_diagnostics(diagnostics);
    diagnostics.in_flight = diagnostics
        .in_flight
        .saturating_sub(expiry.in_flight_entries);
    diagnostics.in_flight_waiters = diagnostics
        .in_flight_waiters
        .saturating_sub(expiry.in_flight_waiters);
    diagnostics.completion_entries = diagnostics
        .completion_entries
        .saturating_sub(expiry.completed_entries);
    diagnostics.completion_bytes = diagnostics
        .completion_bytes
        .saturating_sub(expiry.completed_bytes);
    diagnostics.expired = diagnostics
        .expired
        .saturating_add(expiry.total_entries() as u64);
}

pub(super) fn schedule_entry_expiry(
    expiry_timer: &AssetWorkerExpiryTimer,
    completions: &Arc<CompletionRegistry>,
    diagnostics: &Arc<Mutex<AssetWorkerPoolDiagnostics>>,
    entry: &Arc<CompletionEntry>,
    deadline: Instant,
    generation: u64,
) -> crate::core::CoreResult<crate::core::runtime::tasks::TaskTimerSubscription> {
    let completions = Arc::downgrade(completions);
    let diagnostics = Arc::downgrade(diagnostics);
    let entry = Arc::downgrade(entry);
    expiry_timer.schedule_at(deadline, move || {
        let (Some(completions), Some(diagnostics), Some(entry)) = (
            completions.upgrade(),
            diagnostics.upgrade(),
            entry.upgrade(),
        ) else {
            return;
        };
        expire_scheduled_entry(&completions, &diagnostics, &entry, generation);
    })
}

pub(super) fn maintain_completion_registry(
    completions: &CompletionRegistry,
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
) {
    let expiry = {
        let mut state = lock_completion_registry(completions);
        expire_entries(&mut state, Instant::now())
    };
    record_expiry_for_diagnostics(diagnostics, expiry);
}

impl Drop for AssetWorkerCompletionTicket {
    fn drop(&mut self) {
        let state = lock_completion_registry(&self.completions);
        let is_live_in_flight = state
            .in_flight
            .get(&self.entry.request)
            .is_some_and(|current| Arc::ptr_eq(current, &self.entry));
        if !is_live_in_flight || !self.entry.remove_waiter() {
            return;
        }
        let mut diagnostics = lock_worker_diagnostics(&self.diagnostics);
        diagnostics.in_flight_waiters = diagnostics.in_flight_waiters.saturating_sub(1);
    }
}

impl CompletionEntry {
    pub(super) fn new(request: AssetRequest, request_deadline: Instant) -> Self {
        Self {
            request,
            enqueued_at: Instant::now(),
            request_deadline,
            state: Mutex::new(CompletionEntryState {
                waiters: 1,
                running: false,
                terminal: CompletionTerminal::Pending,
            }),
            expiry_subscription: Mutex::new(None),
            expiry_generation: AtomicU64::new(0),
            changed: Condvar::new(),
        }
    }

    pub(super) fn try_add_waiter(&self, capacity: usize) -> WaiterAdmission {
        let mut state = lock_completion_entry(self);
        let now = Instant::now();
        match &state.terminal {
            CompletionTerminal::Pending if now < self.request_deadline => {}
            CompletionTerminal::Ready { expires_at, .. } if now < *expires_at => {}
            CompletionTerminal::Pending | CompletionTerminal::Ready { .. } => {
                state.terminal = CompletionTerminal::Expired;
                state.running = false;
                self.changed.notify_all();
                return WaiterAdmission::Terminal;
            }
            CompletionTerminal::Cancelled
            | CompletionTerminal::Expired
            | CompletionTerminal::Rejected => return WaiterAdmission::Terminal,
        }
        if state.waiters >= capacity {
            return WaiterAdmission::Full;
        }
        state.waiters = state.waiters.saturating_add(1);
        WaiterAdmission::Added
    }

    fn remove_waiter(&self) -> bool {
        let mut state = lock_completion_entry(self);
        if state.waiters == 0 {
            return false;
        }
        state.waiters = state.waiters.saturating_sub(1);
        true
    }

    pub(super) fn terminate(&self, terminal: CompletionTerminal) -> Option<usize> {
        let mut state = lock_completion_entry(self);
        if !matches!(
            state.terminal,
            CompletionTerminal::Pending | CompletionTerminal::Ready { .. }
        ) {
            return None;
        }
        state.terminal = terminal;
        state.running = false;
        let waiters = state.waiters;
        self.changed.notify_all();
        drop(state);
        self.cancel_expiry();
        Some(waiters)
    }

    pub(super) fn ready(
        &self,
        payload: Arc<CpuAssetPayload>,
        expires_at: Instant,
    ) -> ReadyTransition {
        let mut state = lock_completion_entry(self);
        if !matches!(state.terminal, CompletionTerminal::Pending) {
            return ReadyTransition::Terminal;
        }
        if Instant::now() >= self.request_deadline {
            state.terminal = CompletionTerminal::Expired;
            state.running = false;
            self.changed.notify_all();
            return ReadyTransition::Expired;
        }
        state.terminal = CompletionTerminal::Ready {
            payload,
            expires_at,
        };
        state.running = false;
        self.changed.notify_all();
        ReadyTransition::Ready
    }

    pub(super) fn is_request_expired(&self, now: Instant) -> bool {
        now >= self.request_deadline
    }

    pub(super) fn mark_started(&self, now: Instant) -> Option<Duration> {
        let mut state = lock_completion_entry(self);
        if self.is_request_expired(now) || !matches!(state.terminal, CompletionTerminal::Pending) {
            return None;
        }
        state.running = true;
        Some(now.saturating_duration_since(self.enqueued_at))
    }

    pub(super) fn is_running(&self) -> bool {
        let state = lock_completion_entry(self);
        state.running && matches!(state.terminal, CompletionTerminal::Pending)
    }

    pub(super) fn begin_expiry(&self) -> u64 {
        self.expiry_generation.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub(super) fn install_expiry_subscription(
        &self,
        generation: u64,
        subscription: TaskTimerSubscription,
    ) {
        let previous = {
            let mut current = lock_completion_expiry_subscription(self);
            if self.expiry_generation.load(Ordering::Acquire) != generation {
                Some(subscription)
            } else {
                current.replace(subscription)
            }
        };
        drop(previous);
    }

    pub(super) fn matches_expiry(&self, generation: u64) -> bool {
        self.expiry_generation.load(Ordering::Acquire) == generation
    }

    pub(super) fn cancel_expiry(&self) {
        self.expiry_generation.fetch_add(1, Ordering::AcqRel);
        let subscription = {
            let mut current = lock_completion_expiry_subscription(self);
            current.take()
        };
        drop(subscription);
    }

    pub(super) fn waiter_count(&self) -> usize {
        lock_completion_entry(self).waiters
    }

    pub(super) fn is_completion_expired(&self, now: Instant) -> bool {
        match &lock_completion_entry(self).terminal {
            CompletionTerminal::Ready { expires_at, .. } => now >= *expires_at,
            CompletionTerminal::Expired => true,
            CompletionTerminal::Pending
            | CompletionTerminal::Cancelled
            | CompletionTerminal::Rejected => false,
        }
    }
}

#[derive(Default)]
pub(super) struct ExpiryReport {
    pub(super) in_flight_entries: usize,
    pub(super) in_flight_waiters: usize,
    pub(super) completed_entries: usize,
    pub(super) completed_bytes: usize,
}

impl ExpiryReport {
    pub(super) fn is_empty(&self) -> bool {
        self.total_entries() == 0
    }

    pub(super) fn total_entries(&self) -> usize {
        self.in_flight_entries + self.completed_entries
    }
}

pub(super) enum WaiterAdmission {
    Added,
    Full,
    Terminal,
}

pub(super) enum ReadyTransition {
    Ready,
    Expired,
    Terminal,
}

pub(super) enum AssetWorkerRejectionKind {
    Queue,
    Waiter,
}

fn terminal_result(
    terminal: &CompletionTerminal,
    now: Instant,
) -> Result<Option<Arc<CpuAssetPayload>>, AssetWorkerCompletionError> {
    match terminal {
        CompletionTerminal::Pending => Ok(None),
        CompletionTerminal::Ready {
            payload,
            expires_at,
        } if now < *expires_at => Ok(Some(Arc::clone(payload))),
        CompletionTerminal::Ready { .. } | CompletionTerminal::Expired => {
            Err(AssetWorkerCompletionError::Expired)
        }
        CompletionTerminal::Cancelled => Err(AssetWorkerCompletionError::Cancelled),
        CompletionTerminal::Rejected => Err(AssetWorkerCompletionError::Rejected),
    }
}

fn expire_ticket_terminal(
    state: &mut CompletionEntryState,
    request_deadline: Instant,
    now: Instant,
    changed: &Condvar,
) -> bool {
    let expired = match &state.terminal {
        CompletionTerminal::Pending => now >= request_deadline,
        CompletionTerminal::Ready { expires_at, .. } => now >= *expires_at,
        CompletionTerminal::Cancelled
        | CompletionTerminal::Expired
        | CompletionTerminal::Rejected => false,
    };
    if expired {
        state.terminal = CompletionTerminal::Expired;
        changed.notify_all();
    }
    expired
}

fn unregister_expired_ticket(
    completions: &CompletionRegistry,
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
    entry: &Arc<CompletionEntry>,
) {
    entry.cancel_expiry();
    let mut state = lock_completion_registry(completions);
    let mut expiry = ExpiryReport::default();
    if state
        .in_flight
        .get(&entry.request)
        .is_some_and(|current| Arc::ptr_eq(current, entry))
    {
        state.in_flight.remove(&entry.request);
        expiry.in_flight_entries = 1;
        expiry.in_flight_waiters = entry.waiter_count();
    }
    if state
        .completed
        .get(&entry.request)
        .is_some_and(|completed| Arc::ptr_eq(&completed.entry, entry))
    {
        if let Some(completed) = state.completed.remove(&entry.request) {
            state.completed_bytes = state.completed_bytes.saturating_sub(completed.bytes);
            expiry.completed_entries = 1;
            expiry.completed_bytes = completed.bytes;
        }
    }
    record_expiry_for_diagnostics(diagnostics, expiry);
}

pub(super) fn deadline_after(now: Instant, duration: Duration) -> Instant {
    now.checked_add(duration).unwrap_or(now)
}

pub(super) fn lock_completion_registry(
    completions: &CompletionRegistry,
) -> MutexGuard<'_, CompletionRegistryState> {
    completions
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_completion_entry(entry: &CompletionEntry) -> MutexGuard<'_, CompletionEntryState> {
    entry
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_completion_expiry_subscription(
    entry: &CompletionEntry,
) -> MutexGuard<'_, Option<TaskTimerSubscription>> {
    entry
        .expiry_subscription
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn lock_worker_diagnostics(
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
) -> MutexGuard<'_, AssetWorkerPoolDiagnostics> {
    diagnostics
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn expire_scheduled_entry(
    completions: &CompletionRegistry,
    diagnostics: &Mutex<AssetWorkerPoolDiagnostics>,
    entry: &Arc<CompletionEntry>,
    generation: u64,
) {
    let mut state = lock_completion_registry(completions);
    if !entry.matches_expiry(generation) {
        return;
    }
    let mut expiry = ExpiryReport::default();
    if state
        .in_flight
        .get(&entry.request)
        .is_some_and(|current| Arc::ptr_eq(current, entry))
    {
        state.in_flight.remove(&entry.request);
        expiry.in_flight_entries = 1;
        expiry.in_flight_waiters = entry.waiter_count();
    }
    if state
        .completed
        .get(&entry.request)
        .is_some_and(|completed| Arc::ptr_eq(&completed.entry, entry))
    {
        if let Some(completed) = state.completed.remove(&entry.request) {
            state.completed_bytes = state.completed_bytes.saturating_sub(completed.bytes);
            expiry.completed_entries = 1;
            expiry.completed_bytes = completed.bytes;
        }
    }
    if expiry.is_empty() {
        return;
    }
    entry.terminate(CompletionTerminal::Expired);
    drop(state);
    record_expiry_for_diagnostics(diagnostics, expiry);
}
