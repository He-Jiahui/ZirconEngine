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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LogTailIdentity {
    window_limit: usize,
    first_sequence: Option<u64>,
    last_sequence: Option<u64>,
    record_count: usize,
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
        materialize_tail(&state.records, filter, max_records)
    }

    pub(crate) fn snapshot_tail_if_changed(
        &self,
        filter: &LogFilter,
        max_records: usize,
        known_identity: Option<LogTailIdentity>,
    ) -> (LogTailIdentity, Option<Vec<LogRecord>>) {
        let state = self.lock_state();
        let identity = tail_identity(&state.records, filter, max_records);
        if known_identity == Some(identity) {
            return (identity, None);
        }
        (
            identity,
            Some(materialize_tail(&state.records, filter, max_records)),
        )
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

// Log records are immutable and their sequences are strictly increasing, so these
// boundaries plus the count identify an unchanged bounded matching tail.
fn tail_identity(
    records: &VecDeque<LogRecord>,
    filter: &LogFilter,
    max_records: usize,
) -> LogTailIdentity {
    if max_records == 0 {
        return LogTailIdentity::default();
    }

    let mut identity = LogTailIdentity {
        window_limit: max_records,
        ..LogTailIdentity::default()
    };
    if filter.is_unfiltered() {
        let count = max_records.min(records.len());
        identity.record_count = count;
        identity.first_sequence = records
            .get(records.len().saturating_sub(count))
            .map(LogRecord::sequence);
        identity.last_sequence = records.back().map(LogRecord::sequence);
        return identity;
    }

    for record in records.iter().rev() {
        if !filter.matches(record.entry()) {
            continue;
        }
        identity.last_sequence.get_or_insert(record.sequence());
        identity.first_sequence = Some(record.sequence());
        identity.record_count += 1;
        if identity.record_count == max_records {
            break;
        }
    }
    identity
}

fn materialize_tail(
    records: &VecDeque<LogRecord>,
    filter: &LogFilter,
    max_records: usize,
) -> Vec<LogRecord> {
    if max_records == 0 {
        return Vec::new();
    }

    let mut tail = Vec::with_capacity(max_records.min(records.len()));
    tail.extend(
        records
            .iter()
            .rev()
            .filter(|record| filter.matches(record.entry()))
            .take(max_records)
            .cloned(),
    );
    tail.reverse();
    tail
}
