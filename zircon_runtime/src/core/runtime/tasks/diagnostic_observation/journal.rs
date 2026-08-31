use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::{
    TaskDiagnosticBatch, TaskDiagnosticCursor, TaskDiagnosticIdentity, TaskDiagnosticKind,
    TaskDiagnosticObservation, TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES,
    TASK_DIAGNOSTIC_RETENTION_CAPACITY,
};

static NEXT_TASK_DIAGNOSTIC_SOURCE_ID: AtomicU64 = AtomicU64::new(1);

pub(in crate::core::runtime::tasks) struct TaskDiagnosticJournal {
    source_id: u64,
    state: Mutex<TaskDiagnosticJournalState>,
}

impl Default for TaskDiagnosticJournal {
    fn default() -> Self {
        Self {
            source_id: NEXT_TASK_DIAGNOSTIC_SOURCE_ID.fetch_add(1, Ordering::Relaxed),
            state: Mutex::new(TaskDiagnosticJournalState::default()),
        }
    }
}

impl TaskDiagnosticJournal {
    pub(in crate::core::runtime::tasks) const fn source_id(&self) -> u64 {
        self.source_id
    }

    pub(super) const fn initial_cursor(&self) -> TaskDiagnosticCursor {
        TaskDiagnosticCursor::new(self.source_id, 1)
    }

    pub(in crate::core::runtime::tasks) fn record(
        &self,
        identity: TaskDiagnosticIdentity,
        kind: TaskDiagnosticKind,
        message: Arc<str>,
    ) {
        debug_assert_eq!(identity.scheduler_id(), self.source_id);
        let mut state = self.lock_state();
        let observation_sequence = state.next_observation_sequence;
        state.next_observation_sequence = observation_sequence.saturating_add(1);
        state.entries.push_back(TaskDiagnosticObservation::new(
            observation_sequence,
            identity,
            kind,
            message,
        ));
        if state.entries.len() > TASK_DIAGNOSTIC_RETENTION_CAPACITY {
            state.entries.pop_front();
        }
    }

    pub(super) fn read_after(
        &self,
        cursor: TaskDiagnosticCursor,
        max_entries: usize,
    ) -> TaskDiagnosticBatch {
        let state = self.lock_state();
        let source_changed = cursor.source_id() != self.source_id;
        let oldest_sequence = state
            .entries
            .front()
            .map(TaskDiagnosticObservation::observation_sequence)
            .unwrap_or(state.next_observation_sequence);
        let requested_sequence = if source_changed {
            oldest_sequence
        } else {
            cursor.next_observation_sequence()
        };
        let dropped_count = if source_changed {
            0
        } else {
            oldest_sequence.saturating_sub(requested_sequence)
        };
        let recovered_sequence = requested_sequence.max(oldest_sequence);
        let read_limit = max_entries.min(TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES);
        let observations = state
            .entries
            .iter()
            .filter(|entry| entry.observation_sequence() >= recovered_sequence)
            .take(read_limit)
            .cloned()
            .collect::<Vec<_>>();
        let next_sequence = observations
            .last()
            .map(|entry| entry.observation_sequence().saturating_add(1))
            .unwrap_or(recovered_sequence);
        let has_more = state
            .entries
            .back()
            .is_some_and(|entry| entry.observation_sequence() >= next_sequence);

        TaskDiagnosticBatch::new(
            observations,
            TaskDiagnosticCursor::new(self.source_id, recovered_sequence),
            TaskDiagnosticCursor::new(self.source_id, next_sequence),
            dropped_count,
            source_changed,
            has_more,
        )
    }

    fn lock_state(&self) -> MutexGuard<'_, TaskDiagnosticJournalState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct TaskDiagnosticJournalState {
    next_observation_sequence: u64,
    entries: VecDeque<TaskDiagnosticObservation>,
}

impl Default for TaskDiagnosticJournalState {
    fn default() -> Self {
        Self {
            next_observation_sequence: 1,
            entries: VecDeque::new(),
        }
    }
}
