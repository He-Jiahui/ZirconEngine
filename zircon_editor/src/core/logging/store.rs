use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use super::{EditorLogConfig, EditorLogError, LogEntry, LogFilter, LogRecord};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorLogDiagnostics {
    pub retained_records: usize,
    pub retained_bytes: usize,
    pub dropped_records: u64,
    pub queued_event_records: usize,
    pub queued_event_bytes: usize,
    pub resync_required_records: u64,
    pub event_resyncs: u64,
    pub failed_event_resyncs: u64,
}

pub struct EditorLogStore {
    config: EditorLogConfig,
    state: Mutex<LogStoreState>,
}

#[derive(Default)]
struct LogStoreState {
    next_sequence: u64,
    retained_bytes: usize,
    dropped_records: u64,
    records: VecDeque<LogRecord>,
}

impl EditorLogStore {
    pub fn new(config: EditorLogConfig) -> Self {
        Self {
            config,
            state: Mutex::new(LogStoreState::default()),
        }
    }

    pub fn push(&self, entry: LogEntry) -> Result<LogRecord, EditorLogError> {
        let entry_bytes = entry.estimated_bytes();
        if entry_bytes > self.config.retained_bytes() {
            return Err(EditorLogError::EntryExceedsByteCapacity {
                capacity: self.config.retained_bytes(),
                actual: entry_bytes,
            });
        }
        let mut state = self.lock_state();
        while state.records.len() >= self.config.entry_capacity()
            || state.retained_bytes > self.config.retained_bytes() - entry_bytes
        {
            let removed = state
                .records
                .pop_front()
                .ok_or(EditorLogError::StoreInvariantViolation)?;
            state.retained_bytes -= removed.entry().estimated_bytes();
            state.dropped_records = state.dropped_records.saturating_add(1);
        }
        let sequence = state
            .next_sequence
            .checked_add(1)
            .ok_or(EditorLogError::SequenceExhausted)?;
        state.next_sequence = sequence;
        let record = LogRecord::new(sequence, entry);
        state.retained_bytes += entry_bytes;
        state.records.push_back(record.clone());
        Ok(record)
    }

    pub fn snapshot(&self, filter: &LogFilter) -> Vec<LogRecord> {
        self.lock_state()
            .records
            .iter()
            .filter(|record| filter.matches(record.entry()))
            .cloned()
            .collect()
    }

    pub fn snapshot_tail(&self, filter: &LogFilter, max_records: usize) -> Vec<LogRecord> {
        if max_records == 0 {
            return Vec::new();
        }
        let state = self.lock_state();
        let mut records = Vec::with_capacity(max_records.min(state.records.len()));
        records.extend(
            state
                .records
                .iter()
                .rev()
                .filter(|record| filter.matches(record.entry()))
                .take(max_records)
                .cloned(),
        );
        records.reverse();
        records
    }

    pub fn record(&self, sequence: u64) -> Option<LogRecord> {
        let state = self.lock_state();
        let first_sequence = state.records.front()?.sequence();
        let offset = usize::try_from(sequence.checked_sub(first_sequence)?).ok()?;
        state
            .records
            .get(offset)
            .filter(|record| record.sequence() == sequence)
            .cloned()
    }

    pub(super) fn clear(&self) -> (usize, Option<u64>) {
        let mut state = self.lock_state();
        let cleared = state.records.len();
        let through_sequence = state.records.back().map(LogRecord::sequence);
        state.records.clear();
        state.retained_bytes = 0;
        (cleared, through_sequence)
    }

    pub fn diagnostics(&self) -> EditorLogDiagnostics {
        let state = self.lock_state();
        EditorLogDiagnostics {
            retained_records: state.records.len(),
            retained_bytes: state.retained_bytes,
            dropped_records: state.dropped_records,
            queued_event_records: 0,
            queued_event_bytes: 0,
            resync_required_records: 0,
            event_resyncs: 0,
            failed_event_resyncs: 0,
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, LogStoreState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
