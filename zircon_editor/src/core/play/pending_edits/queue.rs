use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::core::editing::operation::{
    DeferredOperationInvocation, EditOperationTarget, PendingEditRetention,
};
use crate::core::editor_operation::EditorOperationInvocation;

use super::{PendingEditId, PendingEditIntent};

const DEFAULT_MAX_ENTRIES: usize = 4_096;
const DEFAULT_MAX_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_MAX_OLDEST_AGE: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingEditQueueLimits {
    pub max_entries: usize,
    pub max_payload_bytes: usize,
    pub max_oldest_age: Duration,
}

impl Default for PendingEditQueueLimits {
    fn default() -> Self {
        Self {
            max_entries: DEFAULT_MAX_ENTRIES,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
            max_oldest_age: DEFAULT_MAX_OLDEST_AGE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingEditApplyBudget {
    pub max_entries: usize,
    pub max_elapsed: Duration,
}

impl PendingEditApplyBudget {
    pub const fn new(max_entries: usize, max_elapsed: Duration) -> Self {
        Self {
            max_entries,
            max_elapsed,
        }
    }

    pub const fn interactive() -> Self {
        Self::new(128, Duration::from_millis(2))
    }

    pub const fn unlimited() -> Self {
        Self::new(usize::MAX, Duration::MAX)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingEditQueueSummary {
    pub pending_count: usize,
    pub payload_bytes: usize,
    pub oldest_age: Option<Duration>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PendingEditPageCursor(PendingEditId);

impl PendingEditPageCursor {
    pub const fn id(self) -> PendingEditId {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingEditPageEntry {
    pub id: PendingEditId,
    pub target: EditOperationTarget,
    pub operation_id: String,
    pub retention: PendingEditRetention,
    pub payload_bytes: usize,
    pub age: Duration,
    pub retry_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingEditPage {
    pub entries: Vec<PendingEditPageEntry>,
    pub next_cursor: Option<PendingEditPageCursor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingEditEnqueueReport {
    pub id: PendingEditId,
    pub coalesced: bool,
    pub evicted_ids: Vec<PendingEditId>,
    pub summary: PendingEditQueueSummary,
}

#[derive(Debug, Default)]
pub struct PendingEditQueue {
    limits: PendingEditQueueLimits,
    state: std::sync::Mutex<PendingEditQueueState>,
}

#[derive(Debug)]
struct PendingEditQueueState {
    next_id: u64,
    retry_entries: VecDeque<PendingEditIntent>,
    entries: VecDeque<PendingEditIntent>,
    payload_bytes: usize,
    in_flight_count: usize,
}

impl Default for PendingEditQueueState {
    fn default() -> Self {
        Self {
            next_id: 1,
            retry_entries: VecDeque::new(),
            entries: VecDeque::new(),
            payload_bytes: 0,
            in_flight_count: 0,
        }
    }
}

impl PendingEditQueue {
    pub fn new(limits: PendingEditQueueLimits) -> Self {
        Self {
            limits,
            state: std::sync::Mutex::new(PendingEditQueueState::default()),
        }
    }

    pub fn enqueue(
        &self,
        target: EditOperationTarget,
        deferred: DeferredOperationInvocation,
    ) -> Result<PendingEditEnqueueReport, PendingEditQueueError> {
        let registered_target = deferred.target();
        if target != registered_target {
            return Err(PendingEditQueueError::TargetMismatch {
                requested: target,
                registered: registered_target,
            });
        }
        let payload_bytes = serde_json::to_vec(deferred.invocation())
            .map_err(|error| PendingEditQueueError::PayloadEncoding(error.to_string()))?
            .len();
        let retention = deferred.retention().clone();
        let now = Instant::now();
        let mut state = self.lock_state();
        self.reject_expired_queue(&state, now)?;
        if payload_bytes > self.limits.max_payload_bytes {
            return Err(PendingEditQueueError::PayloadLimitExceeded {
                requested_bytes: payload_bytes,
                max_payload_bytes: self.limits.max_payload_bytes,
            });
        }

        if retention.is_latest() {
            if let Some(location) = state.find_cohort(target, deferred.invocation(), &retention) {
                let (existing_payload_bytes, id) = {
                    let existing = state
                        .intent_at(location)
                        .ok_or(PendingEditQueueError::QueueStateInconsistent)?;
                    (existing.payload_bytes(), existing.id)
                };
                let resulting_payload_bytes = state
                    .payload_bytes
                    .checked_sub(existing_payload_bytes)
                    .and_then(|bytes| bytes.checked_add(payload_bytes))
                    .ok_or(PendingEditQueueError::PayloadAccountingOverflow)?;
                self.reject_payload_limit(resulting_payload_bytes)?;
                state
                    .intent_at_mut(location)
                    .ok_or(PendingEditQueueError::QueueStateInconsistent)?
                    .replace(target, deferred, payload_bytes, now);
                state.payload_bytes = resulting_payload_bytes;
                return Ok(PendingEditEnqueueReport {
                    id,
                    coalesced: true,
                    evicted_ids: Vec::new(),
                    summary: Self::summary_from_state(&state, now),
                });
            }
        }

        let mut eviction = None;
        if let Some(limits) = retention.bounded_limits() {
            if payload_bytes > limits.max_payload_bytes().get() {
                return Err(PendingEditQueueError::RetentionPayloadLimitExceeded {
                    requested_bytes: payload_bytes,
                    max_payload_bytes: limits.max_payload_bytes().get(),
                });
            }
            let cohort_count = state.cohort_count(target, deferred.invocation(), &retention);
            if let Some(existing) = state.find_cohort(target, deferred.invocation(), &retention) {
                if state
                    .intent_at(existing)
                    .ok_or(PendingEditQueueError::QueueStateInconsistent)?
                    .retention
                    != retention
                {
                    return Err(PendingEditQueueError::RetentionPolicyConflict);
                }
            }
            if cohort_count >= limits.max_entries().get() {
                eviction = state.oldest_cohort(target, deferred.invocation(), &retention);
            }
        }

        let evicted_payload_bytes = match eviction {
            Some(location) => state
                .intent_at(location)
                .ok_or(PendingEditQueueError::QueueStateInconsistent)?
                .payload_bytes(),
            None => 0,
        };
        let resulting_entries = state
            .len()
            .checked_sub(usize::from(eviction.is_some()))
            .and_then(|entries| entries.checked_add(1))
            .ok_or(PendingEditQueueError::PayloadAccountingOverflow)?;
        if resulting_entries > self.limits.max_entries {
            return Err(PendingEditQueueError::EntryLimitReached);
        }
        let resulting_payload_bytes = state
            .payload_bytes
            .checked_sub(evicted_payload_bytes)
            .and_then(|bytes| bytes.checked_add(payload_bytes))
            .ok_or(PendingEditQueueError::PayloadAccountingOverflow)?;
        self.reject_payload_limit(resulting_payload_bytes)?;

        let id = PendingEditId::new(state.next_id);
        state.next_id = state
            .next_id
            .checked_add(1)
            .ok_or(PendingEditQueueError::IdentifierExhausted)?;
        let mut evicted_ids = Vec::new();
        if let Some(location) = eviction {
            let evicted = state
                .remove(location)
                .ok_or(PendingEditQueueError::QueueStateInconsistent)?;
            evicted_ids.push(evicted.id);
        }
        state.entries.push_back(PendingEditIntent::new(
            id,
            target,
            deferred,
            payload_bytes,
            now,
        ));
        state.payload_bytes = resulting_payload_bytes;
        Ok(PendingEditEnqueueReport {
            id,
            coalesced: false,
            evicted_ids,
            summary: Self::summary_from_state(&state, now),
        })
    }

    pub fn summary(&self) -> PendingEditQueueSummary {
        let state = self.lock_state();
        Self::summary_from_state(&state, Instant::now())
    }

    pub fn is_empty(&self) -> bool {
        self.summary().pending_count == 0
    }

    /// Returns compact records only. The invocation payload stays shared in the
    /// queue and is intentionally not cloned into a UI-facing page.
    pub fn page(&self, after: Option<PendingEditPageCursor>, limit: usize) -> PendingEditPage {
        let state = self.lock_state();
        let now = Instant::now();
        let limit = limit.clamp(1, 128);
        let mut candidates = state
            .retry_entries
            .iter()
            .chain(state.entries.iter())
            .filter(|intent| after.is_none_or(|cursor| intent.id > cursor.id()));
        let page_capacity = candidates.clone().count().min(limit);
        let mut entries = Vec::with_capacity(page_capacity);
        for intent in candidates.by_ref().take(limit) {
            entries.push(PendingEditPageEntry {
                id: intent.id,
                target: intent.target,
                operation_id: intent.invocation.operation_id.to_string(),
                retention: intent.retention.clone(),
                payload_bytes: intent.payload_bytes(),
                age: now.saturating_duration_since(intent.enqueued_at()),
                retry_count: intent.retry_count(),
            });
        }
        let next_cursor = candidates
            .next()
            .and(entries.last())
            .map(|entry| PendingEditPageCursor(entry.id));
        PendingEditPage {
            entries,
            next_cursor,
        }
    }

    pub fn decision_prompt(&self) -> Option<super::PendingEditDecisionPrompt> {
        let summary = self.summary();
        (summary.pending_count != 0).then(|| super::PendingEditDecisionPrompt::new(summary))
    }

    pub fn apply_with_budget<E>(
        &self,
        budget: PendingEditApplyBudget,
        mut apply: impl FnMut(&PendingEditIntent) -> Result<(), E>,
    ) -> super::PendingEditApplyReport<E> {
        let started_at = Instant::now();
        let (mut retry_remaining, mut pending_remaining) = {
            let state = self.lock_state();
            (state.retry_entries.len(), state.entries.len())
        };
        let mut applied = Vec::new();
        let mut failures = Vec::new();

        while applied.len().saturating_add(failures.len()) < budget.max_entries
            && started_at.elapsed() < budget.max_elapsed
        {
            let Some(mut intent) = self.take_next(&mut retry_remaining, &mut pending_remaining)
            else {
                break;
            };
            let outcome = catch_unwind(AssertUnwindSafe(|| apply(&intent)));
            let payload_bytes = intent.payload_bytes();
            match outcome {
                Ok(Ok(())) => {
                    self.complete_in_flight(payload_bytes);
                    applied.push(intent.id);
                }
                Ok(Err(error)) => {
                    intent.mark_retry();
                    failures.push(super::PendingEditApplyFailure {
                        intent: intent.clone(),
                        error,
                    });
                    self.requeue_for_retry(intent);
                }
                Err(panic) => {
                    intent.mark_retry();
                    self.requeue_for_retry(intent);
                    resume_unwind(panic);
                }
            }
        }

        let remaining = self.summary();
        let attempts = applied.len().saturating_add(failures.len());
        super::PendingEditApplyReport {
            applied,
            failures,
            budget_exhausted: remaining.pending_count != 0
                && (attempts >= budget.max_entries || started_at.elapsed() >= budget.max_elapsed),
            remaining,
        }
    }

    pub fn discard(&self) -> super::PendingEditDiscardReport {
        let mut state = self.lock_state();
        let discarded_count = state.len();
        let discarded_payload_bytes = state.payload_bytes;
        state.retry_entries.clear();
        state.entries.clear();
        if state.in_flight_count == 0 {
            state.payload_bytes = 0;
        }
        super::PendingEditDiscardReport {
            discarded_count,
            discarded_payload_bytes,
            remaining: Self::summary_from_state(&state, Instant::now()),
        }
    }

    fn take_next(
        &self,
        retry_remaining: &mut usize,
        pending_remaining: &mut usize,
    ) -> Option<PendingEditIntent> {
        let mut state = self.lock_state();
        let next = if *retry_remaining != 0 {
            *retry_remaining -= 1;
            state.retry_entries.pop_front()
        } else if *pending_remaining != 0 {
            *pending_remaining -= 1;
            state.entries.pop_front()
        } else {
            None
        };
        if let Some(intent) = next.as_ref() {
            state.in_flight_count = state.in_flight_count.saturating_add(1);
        }
        next
    }

    fn requeue_for_retry(&self, intent: PendingEditIntent) {
        let mut state = self.lock_state();
        state.in_flight_count = state.in_flight_count.saturating_sub(1);
        state.retry_entries.push_back(intent);
    }

    fn complete_in_flight(&self, payload_bytes: usize) {
        let mut state = self.lock_state();
        state.in_flight_count = state.in_flight_count.saturating_sub(1);
        state.payload_bytes = state.payload_bytes.saturating_sub(payload_bytes);
    }

    fn reject_expired_queue(
        &self,
        state: &PendingEditQueueState,
        now: Instant,
    ) -> Result<(), PendingEditQueueError> {
        let Some(oldest) = state.oldest_enqueued_at() else {
            return Ok(());
        };
        let age = now.saturating_duration_since(oldest);
        if age > self.limits.max_oldest_age {
            return Err(PendingEditQueueError::OldestAgeExceeded {
                oldest_age: age,
                max_oldest_age: self.limits.max_oldest_age,
            });
        }
        if let Some((intent, age, max_age)) = state
            .retry_entries
            .iter()
            .chain(state.entries.iter())
            .find_map(|intent| {
                let limits = intent.retention.bounded_limits()?;
                let age = now.saturating_duration_since(intent.enqueued_at());
                (age > limits.max_age()).then_some((intent, age, limits.max_age()))
            })
        {
            return Err(PendingEditQueueError::RetentionAgeExceeded {
                operation_id: intent.invocation.operation_id.to_string(),
                oldest_age: age,
                max_oldest_age: max_age,
            });
        }
        Ok(())
    }

    fn reject_payload_limit(&self, payload_bytes: usize) -> Result<(), PendingEditQueueError> {
        if payload_bytes > self.limits.max_payload_bytes {
            return Err(PendingEditQueueError::PayloadLimitExceeded {
                requested_bytes: payload_bytes,
                max_payload_bytes: self.limits.max_payload_bytes,
            });
        }
        Ok(())
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, PendingEditQueueState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn summary_from_state(state: &PendingEditQueueState, now: Instant) -> PendingEditQueueSummary {
        PendingEditQueueSummary {
            pending_count: state.len(),
            payload_bytes: state.payload_bytes,
            oldest_age: state
                .oldest_enqueued_at()
                .map(|oldest| now.saturating_duration_since(oldest)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PendingEditLocation {
    Retry(usize),
    Pending(usize),
}

impl PendingEditQueueState {
    fn len(&self) -> usize {
        self.retry_entries
            .len()
            .saturating_add(self.entries.len())
            .saturating_add(self.in_flight_count)
    }

    fn oldest_enqueued_at(&self) -> Option<Instant> {
        self.retry_entries
            .iter()
            .chain(self.entries.iter())
            .map(PendingEditIntent::enqueued_at)
            .min()
    }

    fn find_cohort(
        &self,
        target: EditOperationTarget,
        invocation: &EditorOperationInvocation,
        retention: &PendingEditRetention,
    ) -> Option<PendingEditLocation> {
        self.retry_entries
            .iter()
            .position(|intent| intent.belongs_to_cohort(target, invocation, retention))
            .map(PendingEditLocation::Retry)
            .or_else(|| {
                self.entries
                    .iter()
                    .position(|intent| intent.belongs_to_cohort(target, invocation, retention))
                    .map(PendingEditLocation::Pending)
            })
    }

    fn cohort_count(
        &self,
        target: EditOperationTarget,
        invocation: &EditorOperationInvocation,
        retention: &PendingEditRetention,
    ) -> usize {
        self.retry_entries
            .iter()
            .chain(self.entries.iter())
            .filter(|intent| intent.belongs_to_cohort(target, invocation, retention))
            .count()
    }

    fn oldest_cohort(
        &self,
        target: EditOperationTarget,
        invocation: &EditorOperationInvocation,
        retention: &PendingEditRetention,
    ) -> Option<PendingEditLocation> {
        let retry = self
            .retry_entries
            .iter()
            .enumerate()
            .filter(|(_, intent)| intent.belongs_to_cohort(target, invocation, retention))
            .map(|(index, intent)| (PendingEditLocation::Retry(index), intent));
        let pending = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, intent)| intent.belongs_to_cohort(target, invocation, retention))
            .map(|(index, intent)| (PendingEditLocation::Pending(index), intent));
        retry
            .chain(pending)
            .min_by_key(|(_, intent)| (intent.enqueued_at(), intent.id))
            .map(|(location, _)| location)
    }

    fn intent_at(&self, location: PendingEditLocation) -> Option<&PendingEditIntent> {
        match location {
            PendingEditLocation::Retry(index) => self.retry_entries.get(index),
            PendingEditLocation::Pending(index) => self.entries.get(index),
        }
    }

    fn intent_at_mut(&mut self, location: PendingEditLocation) -> Option<&mut PendingEditIntent> {
        match location {
            PendingEditLocation::Retry(index) => self.retry_entries.get_mut(index),
            PendingEditLocation::Pending(index) => self.entries.get_mut(index),
        }
    }

    fn remove(&mut self, location: PendingEditLocation) -> Option<PendingEditIntent> {
        match location {
            PendingEditLocation::Retry(index) => self.retry_entries.remove(index),
            PendingEditLocation::Pending(index) => self.entries.remove(index),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingEditQueueError {
    IdentifierExhausted,
    EntryLimitReached,
    PayloadLimitExceeded {
        requested_bytes: usize,
        max_payload_bytes: usize,
    },
    RetentionPayloadLimitExceeded {
        requested_bytes: usize,
        max_payload_bytes: usize,
    },
    OldestAgeExceeded {
        oldest_age: Duration,
        max_oldest_age: Duration,
    },
    RetentionAgeExceeded {
        operation_id: String,
        oldest_age: Duration,
        max_oldest_age: Duration,
    },
    RetentionPolicyConflict,
    PayloadEncoding(String),
    PayloadAccountingOverflow,
    QueueStateInconsistent,
    TargetMismatch {
        requested: EditOperationTarget,
        registered: EditOperationTarget,
    },
}

impl Display for PendingEditQueueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdentifierExhausted => {
                formatter.write_str("pending edit identifier space is exhausted")
            }
            Self::EntryLimitReached => formatter.write_str(
                "pending edit queue reached its entry limit; resolve or discard queued edits first",
            ),
            Self::PayloadLimitExceeded {
                requested_bytes,
                max_payload_bytes,
            } => write!(
                formatter,
                "pending edit payload requires {requested_bytes} bytes but the queue limit is {max_payload_bytes}"
            ),
            Self::RetentionPayloadLimitExceeded {
                requested_bytes,
                max_payload_bytes,
            } => write!(
                formatter,
                "pending edit payload requires {requested_bytes} bytes but its operation retention limit is {max_payload_bytes}"
            ),
            Self::OldestAgeExceeded {
                oldest_age,
                max_oldest_age,
            } => write!(
                formatter,
                "oldest pending edit is {oldest_age:?}, above the configured {max_oldest_age:?} limit"
            ),
            Self::RetentionAgeExceeded {
                operation_id,
                oldest_age,
                max_oldest_age,
            } => write!(
                formatter,
                "oldest pending {operation_id} edit is {oldest_age:?}, above its retention limit of {max_oldest_age:?}"
            ),
            Self::RetentionPolicyConflict => formatter.write_str(
                "pending edit retention policy conflicts with an existing coalescing cohort",
            ),
            Self::PayloadEncoding(error) => {
                write!(formatter, "failed to measure pending edit payload: {error}")
            }
            Self::PayloadAccountingOverflow => {
                formatter.write_str("pending edit payload accounting overflowed")
            }
            Self::QueueStateInconsistent => formatter
                .write_str("pending edit queue changed while an operation was being admitted"),
            Self::TargetMismatch {
                requested,
                registered,
            } => write!(
                formatter,
                "pending edit target {requested:?} does not match its registered target {registered:?}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::*;

    const PAGE_BUFFER_SAMPLE_PAIRS: usize = 17;
    const PAGE_BUFFER_MEASURE_ITERATIONS: usize = 4_096;
    const PAGE_BUFFER_SIZE: usize = 128;

    fn elapsed_micros(run: impl FnOnce()) -> u128 {
        let started = Instant::now();
        run();
        started.elapsed().as_micros()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let rank = (samples.len() * 95).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    fn legacy_page_buffer(values: &[usize]) -> Vec<usize> {
        values
            .iter()
            .filter(|_| true)
            .take(PAGE_BUFFER_SIZE)
            .copied()
            .collect()
    }

    fn reserved_page_buffer(values: &[usize]) -> Vec<usize> {
        let mut candidates = values.iter().filter(|_| true);
        let page_capacity = candidates
            .size_hint()
            .1
            .unwrap_or(PAGE_BUFFER_SIZE)
            .min(PAGE_BUFFER_SIZE);
        let mut page = Vec::with_capacity(page_capacity);
        for value in candidates.by_ref().take(PAGE_BUFFER_SIZE) {
            page.push(*value);
        }
        page
    }

    #[test]
    fn removing_a_stale_location_fails_without_panicking() {
        let mut state = PendingEditQueueState::default();

        assert!(state.intent_at(PendingEditLocation::Pending(0)).is_none());
        assert!(state.intent_at_mut(PendingEditLocation::Retry(0)).is_none());
        assert!(state.remove(PendingEditLocation::Pending(0)).is_none());
        assert!(state.remove(PendingEditLocation::Retry(0)).is_none());
    }

    #[test]
    #[ignore = "release performance evidence for the managed validation coordinator"]
    fn optimization_batch_20260826b_editor07_pending_edit_page_capacity_performance_evidence() {
        let values = (0..PAGE_BUFFER_SIZE).collect::<Vec<_>>();

        for _ in 0..4 {
            assert_eq!(
                black_box(legacy_page_buffer(&values)).len(),
                PAGE_BUFFER_SIZE
            );
            assert_eq!(
                black_box(reserved_page_buffer(&values)).len(),
                PAGE_BUFFER_SIZE
            );
        }

        let mut legacy_samples = Vec::with_capacity(PAGE_BUFFER_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(PAGE_BUFFER_SAMPLE_PAIRS);
        for sample_index in 0..PAGE_BUFFER_SAMPLE_PAIRS {
            let measure_legacy = || {
                elapsed_micros(|| {
                    for _ in 0..PAGE_BUFFER_MEASURE_ITERATIONS {
                        black_box(legacy_page_buffer(black_box(&values)));
                    }
                })
            };
            let measure_optimized = || {
                elapsed_micros(|| {
                    for _ in 0..PAGE_BUFFER_MEASURE_ITERATIONS {
                        black_box(reserved_page_buffer(black_box(&values)));
                    }
                })
            };
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
        let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
        println!(
            "EDITOR07_PENDING_EDIT_PAGE_CAPACITY_BENCH_V1 sample_pairs={} page_entries={} iterations_per_sample={} legacy_page_buffer_initial_capacity=0 optimized_page_buffer_initial_capacity={} legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
            PAGE_BUFFER_SAMPLE_PAIRS,
            PAGE_BUFFER_SIZE,
            PAGE_BUFFER_MEASURE_ITERATIONS,
            PAGE_BUFFER_SIZE,
            legacy_p95,
            optimized_p95,
            legacy_samples,
            optimized_samples,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
            "reserved pending-edit page buffer p95 must be at least 20% below progressive growth: legacy={legacy_p95}us optimized={optimized_p95}us"
        );
    }
}

impl std::error::Error for PendingEditQueueError {}
