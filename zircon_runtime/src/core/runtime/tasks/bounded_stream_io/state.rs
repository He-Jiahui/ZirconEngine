use std::collections::VecDeque;
use std::mem;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::decoder::DecodedLine;
use super::model::{
    BoundedStreamIoBatch, BoundedStreamIoDiagnostics, BoundedStreamIoDrainBudget,
    BoundedStreamIoFailure, BoundedStreamIoLimits, BoundedStreamIoRecord, BoundedStreamIoStreamId,
};

const MAX_READ_FAILURE_MESSAGE_BYTES: usize = 4 * 1024;

pub(super) struct CaptureState {
    inner: Mutex<CaptureStateInner>,
    terminal: Condvar,
}

struct CaptureStateInner {
    accepting_records: bool,
    cancellation_requested: bool,
    queue: VecDeque<BoundedStreamIoRecord>,
    queued_bytes: usize,
    peak_queued_records: usize,
    peak_queued_bytes: usize,
    dropped_records: u64,
    dropped_bytes: u64,
    truncated_records: u64,
    truncated_bytes: u64,
    lossy_utf8_records: u64,
    active_readers: usize,
    completed_readers: u64,
    cancelled_readers: u64,
    failed_readers: u64,
    failures: Vec<BoundedStreamIoFailure>,
}

pub(super) enum ReaderOutcome {
    Completed,
    Cancelled,
    Failed(BoundedStreamIoFailure),
}

impl CaptureState {
    pub fn new(reader_count: usize) -> Self {
        Self {
            inner: Mutex::new(CaptureStateInner {
                accepting_records: true,
                cancellation_requested: false,
                queue: VecDeque::new(),
                queued_bytes: 0,
                peak_queued_records: 0,
                peak_queued_bytes: 0,
                dropped_records: 0,
                dropped_bytes: 0,
                truncated_records: 0,
                truncated_bytes: 0,
                lossy_utf8_records: 0,
                active_readers: reader_count,
                completed_readers: 0,
                cancelled_readers: 0,
                failed_readers: 0,
                failures: Vec::with_capacity(reader_count),
            }),
            terminal: Condvar::new(),
        }
    }

    pub fn enqueue(
        &self,
        stream: &BoundedStreamIoStreamId,
        line: DecodedLine,
        limits: BoundedStreamIoLimits,
    ) -> bool {
        let mut state = self.lock();
        if !state.accepting_records {
            return false;
        }

        if line.truncated_bytes > 0 {
            state.truncated_records = state.truncated_records.saturating_add(1);
            state.truncated_bytes = state.truncated_bytes.saturating_add(line.truncated_bytes);
        }
        if line.lossy_utf8 {
            state.lossy_utf8_records = state.lossy_utf8_records.saturating_add(1);
        }

        let retained_bytes = line
            .text
            .capacity()
            .saturating_add(mem::size_of::<BoundedStreamIoRecord>());
        let next_bytes = state.queued_bytes.checked_add(retained_bytes);
        if state.queue.len() >= limits.queue_entry_capacity
            || next_bytes.is_none_or(|bytes| bytes > limits.queue_byte_capacity)
        {
            state.dropped_records = state.dropped_records.saturating_add(1);
            state.dropped_bytes = state.dropped_bytes.saturating_add(line.source_bytes);
            return true;
        }

        state.queued_bytes = next_bytes.unwrap_or(state.queued_bytes);
        state.queue.push_back(BoundedStreamIoRecord {
            stream: stream.clone(),
            text: line.text,
            truncated_bytes: line.truncated_bytes,
            lossy_utf8: line.lossy_utf8,
            captured_at: Instant::now(),
            retained_bytes,
        });
        state.peak_queued_records = state.peak_queued_records.max(state.queue.len());
        state.peak_queued_bytes = state.peak_queued_bytes.max(state.queued_bytes);
        true
    }

    pub fn finish_reader(&self, outcome: ReaderOutcome) {
        let mut state = self.lock();
        state.active_readers = state.active_readers.saturating_sub(1);
        match outcome {
            ReaderOutcome::Completed => {
                state.completed_readers = state.completed_readers.saturating_add(1);
            }
            ReaderOutcome::Cancelled => {
                state.cancelled_readers = state.cancelled_readers.saturating_add(1);
            }
            ReaderOutcome::Failed(mut failure) => {
                truncate_utf8(&mut failure.message, MAX_READ_FAILURE_MESSAGE_BYTES);
                state.failed_readers = state.failed_readers.saturating_add(1);
                state.failures.push(failure);
            }
        }
        if state.active_readers == 0 {
            self.terminal.notify_all();
        }
    }

    pub fn close_consumer(&self) {
        let mut state = self.lock();
        state.accepting_records = false;
        state.cancellation_requested = true;
        state.queue.clear();
        state.queued_bytes = 0;
    }

    pub fn request_cancellation(&self) {
        self.lock().cancellation_requested = true;
    }

    pub fn is_cancellation_requested(&self) -> bool {
        self.lock().cancellation_requested
    }

    pub fn accepts_records(&self) -> bool {
        self.lock().accepting_records
    }

    pub fn wait_until_terminal(&self, timeout: Duration) -> bool {
        let state = self.lock();
        if state.active_readers == 0 {
            return true;
        }
        let (state, _) = self
            .terminal
            .wait_timeout_while(state, timeout, |state| state.active_readers != 0)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.active_readers == 0
    }

    pub fn drain(&self, budget: BoundedStreamIoDrainBudget) -> BoundedStreamIoBatch {
        let started = Instant::now();
        let mut state = self.lock();
        let oldest_age = state
            .queue
            .front()
            .map_or(Duration::ZERO, |record| record.captured_at.elapsed());
        let mut records = Vec::with_capacity(budget.max_records.min(state.queue.len()));
        let mut drained_bytes = 0usize;

        while records.len() < budget.max_records && started.elapsed() < budget.max_time {
            let Some(next) = state.queue.front() else {
                break;
            };
            if !records.is_empty()
                && drained_bytes.saturating_add(next.retained_bytes) > budget.max_bytes
            {
                break;
            }
            let Some(record) = state.queue.pop_front() else {
                break;
            };
            state.queued_bytes = state.queued_bytes.saturating_sub(record.retained_bytes);
            drained_bytes = drained_bytes.saturating_add(record.retained_bytes);
            records.push(record);
        }

        let diagnostics = diagnostics_from(&state);
        BoundedStreamIoBatch {
            records,
            drained_bytes,
            oldest_age,
            elapsed: started.elapsed(),
            diagnostics,
        }
    }

    pub fn diagnostics(&self) -> BoundedStreamIoDiagnostics {
        diagnostics_from(&self.lock())
    }

    pub fn failures(&self) -> Vec<BoundedStreamIoFailure> {
        self.lock().failures.clone()
    }

    fn lock(&self) -> MutexGuard<'_, CaptureStateInner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn diagnostics_from(state: &CaptureStateInner) -> BoundedStreamIoDiagnostics {
    BoundedStreamIoDiagnostics {
        queued_records: state.queue.len(),
        queued_bytes: state.queued_bytes,
        peak_queued_records: state.peak_queued_records,
        peak_queued_bytes: state.peak_queued_bytes,
        dropped_records: state.dropped_records,
        dropped_bytes: state.dropped_bytes,
        truncated_records: state.truncated_records,
        truncated_bytes: state.truncated_bytes,
        lossy_utf8_records: state.lossy_utf8_records,
        completed_readers: state.completed_readers,
        cancelled_readers: state.cancelled_readers,
        failed_readers: state.failed_readers,
        active_readers: state.active_readers,
    }
}

fn truncate_utf8(value: &mut String, byte_limit: usize) {
    if value.len() <= byte_limit {
        return;
    }
    let mut end = byte_limit;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
}
