use std::io::{self, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::super::super::worker::{DurableOutput, SINK_THREAD_NAME};
use super::validation::LineValidation;

#[derive(Clone)]
pub(super) struct InstrumentedSlowOutput {
    counters: Arc<OutputCounters>,
}

struct OutputCounters {
    write_delay: Duration,
    write_calls: AtomicU64,
    flush_calls: AtomicU64,
    sync_calls: AtomicU64,
    written_bytes: AtomicU64,
    wrong_thread_calls: AtomicU64,
    validation: Mutex<LineValidation>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OutputSnapshot {
    pub(super) write_calls: u64,
    pub(super) flush_calls: u64,
    pub(super) sync_calls: u64,
    pub(super) written_bytes: u64,
    pub(super) wrong_thread_calls: u64,
    pub(super) validated_records: u64,
    pub(super) duplicate_records: u64,
    pub(super) malformed_records: u64,
}

impl InstrumentedSlowOutput {
    pub(super) fn new(write_delay: Duration, expected_records: usize) -> Self {
        Self {
            counters: Arc::new(OutputCounters {
                write_delay,
                write_calls: AtomicU64::new(0),
                flush_calls: AtomicU64::new(0),
                sync_calls: AtomicU64::new(0),
                written_bytes: AtomicU64::new(0),
                wrong_thread_calls: AtomicU64::new(0),
                validation: Mutex::new(LineValidation::new(expected_records)),
            }),
        }
    }

    pub(super) fn snapshot(&self) -> OutputSnapshot {
        let validation = self.counters.validation.lock().unwrap().snapshot();
        OutputSnapshot {
            write_calls: self.counters.write_calls.load(Ordering::Relaxed),
            flush_calls: self.counters.flush_calls.load(Ordering::Relaxed),
            sync_calls: self.counters.sync_calls.load(Ordering::Relaxed),
            written_bytes: self.counters.written_bytes.load(Ordering::Relaxed),
            wrong_thread_calls: self.counters.wrong_thread_calls.load(Ordering::Relaxed),
            validated_records: validation.records,
            duplicate_records: validation.duplicates,
            malformed_records: validation.malformed,
        }
    }

    fn record_thread(&self) {
        if std::thread::current().name() != Some(SINK_THREAD_NAME) {
            self.counters
                .wrong_thread_calls
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Write for InstrumentedSlowOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.record_thread();
        if !self.counters.write_delay.is_zero() {
            std::thread::sleep(self.counters.write_delay);
        }
        self.counters.write_calls.fetch_add(1, Ordering::Relaxed);
        self.counters
            .written_bytes
            .fetch_add(buffer.len() as u64, Ordering::Relaxed);
        self.counters.validation.lock().unwrap().observe(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.record_thread();
        self.counters.flush_calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl DurableOutput for InstrumentedSlowOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        self.record_thread();
        self.counters.sync_calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
