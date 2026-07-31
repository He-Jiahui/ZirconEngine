use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

use crate::core::editing::operation::{DeferredOperationInvocation, PendingEditRetention};
use crate::core::editor_operation::EditorOperationInvocation;

use super::super::PlayEditTarget;
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
    pub target: PlayEditTarget,
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
}

impl Default for PendingEditQueueState {
    fn default() -> Self {
        Self {
            next_id: 1,
            retry_entries: VecDeque::new(),
            entries: VecDeque::new(),
            payload_bytes: 0,
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
        target: PlayEditTarget,
        deferred: DeferredOperationInvocation,
    ) -> Result<PendingEditEnqueueReport, PendingEditQueueError> {
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
                let existing = state.intent_at(location);
                let resulting_payload_bytes = state
                    .payload_bytes
                    .checked_sub(existing.payload_bytes())
                    .and_then(|bytes| bytes.checked_add(payload_bytes))
                    .ok_or(PendingEditQueueError::PayloadAccountingOverflow)?;
                self.reject_payload_limit(resulting_payload_bytes)?;
                let id = existing.id;
                state
                    .intent_at_mut(location)
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
                if state.intent_at(existing).retention != retention {
                    return Err(PendingEditQueueError::RetentionPolicyConflict);
                }
            }
            if cohort_count >= limits.max_entries().get() {
                eviction = state.oldest_cohort(target, deferred.invocation(), &retention);
            }
        }

        let evicted_payload_bytes = eviction
            .map(|location| state.intent_at(location).payload_bytes())
            .unwrap_or(0);
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
            evicted_ids.push(state.remove(location).id);
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
        let entries = candidates
            .by_ref()
            .take(limit)
            .map(|intent| PendingEditPageEntry {
                id: intent.id,
                target: intent.target,
                operation_id: intent.invocation.operation_id.to_string(),
                retention: intent.retention.clone(),
                payload_bytes: intent.payload_bytes(),
                age: now.saturating_duration_since(intent.enqueued_at()),
                retry_count: intent.retry_count(),
            })
            .collect::<Vec<_>>();
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
        let mut retry_remaining = self.lock_state().retry_entries.len();
        let mut applied = Vec::new();
        let mut failures = Vec::new();

        while applied.len().saturating_add(failures.len()) < budget.max_entries
            && started_at.elapsed() < budget.max_elapsed
        {
            let Some(mut intent) = self.take_next(&mut retry_remaining) else {
                break;
            };
            match apply(&intent) {
                Ok(()) => applied.push(intent.id),
                Err(error) => {
                    intent.mark_retry();
                    failures.push(super::PendingEditApplyFailure {
                        intent: intent.clone(),
                        error,
                    });
                    self.requeue_for_retry(intent);
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
        state.payload_bytes = 0;
        super::PendingEditDiscardReport {
            discarded_count,
            discarded_payload_bytes,
            remaining: Self::summary_from_state(&state, Instant::now()),
        }
    }

    fn take_next(&self, retry_remaining: &mut usize) -> Option<PendingEditIntent> {
        let mut state = self.lock_state();
        let next = if *retry_remaining != 0 {
            *retry_remaining -= 1;
            state.retry_entries.pop_front()
        } else {
            state.entries.pop_front()
        };
        if let Some(intent) = next.as_ref() {
            state.payload_bytes = state.payload_bytes.saturating_sub(intent.payload_bytes());
        }
        next
    }

    fn requeue_for_retry(&self, intent: PendingEditIntent) {
        let mut state = self.lock_state();
        state.payload_bytes = state.payload_bytes.saturating_add(intent.payload_bytes());
        state.retry_entries.push_back(intent);
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
        if let Some((intent, age)) = state
            .retry_entries
            .iter()
            .chain(state.entries.iter())
            .map(|intent| (intent, now.saturating_duration_since(intent.enqueued_at())))
            .find(|(intent, age)| {
                intent
                    .retention
                    .bounded_limits()
                    .is_some_and(|limits| *age > limits.max_age())
            })
        {
            return Err(PendingEditQueueError::RetentionAgeExceeded {
                operation_id: intent.invocation.operation_id.to_string(),
                oldest_age: age,
                max_oldest_age: intent
                    .retention
                    .bounded_limits()
                    .expect("bounded intent must retain its limits")
                    .max_age(),
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
        self.retry_entries.len().saturating_add(self.entries.len())
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
        target: PlayEditTarget,
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
        target: PlayEditTarget,
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
        target: PlayEditTarget,
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

    fn intent_at(&self, location: PendingEditLocation) -> &PendingEditIntent {
        match location {
            PendingEditLocation::Retry(index) => &self.retry_entries[index],
            PendingEditLocation::Pending(index) => &self.entries[index],
        }
    }

    fn intent_at_mut(&mut self, location: PendingEditLocation) -> &mut PendingEditIntent {
        match location {
            PendingEditLocation::Retry(index) => &mut self.retry_entries[index],
            PendingEditLocation::Pending(index) => &mut self.entries[index],
        }
    }

    fn remove(&mut self, location: PendingEditLocation) -> PendingEditIntent {
        match location {
            PendingEditLocation::Retry(index) => self.retry_entries.remove(index),
            PendingEditLocation::Pending(index) => self.entries.remove(index),
        }
        .expect("queue location must resolve while the state lock is held")
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
        }
    }
}

impl std::error::Error for PendingEditQueueError {}
