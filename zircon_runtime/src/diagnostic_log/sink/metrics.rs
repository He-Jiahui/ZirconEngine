use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::diagnostic_log::DiagnosticLogLevel;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticLogSinkSnapshot {
    pub queue_depth: usize,
    pub max_queue_depth: usize,
    pub dequeued_records: u64,
    pub written_records: u64,
    pub written_bytes: u64,
    pub flush_batches: u64,
    pub max_queue_age: Duration,
    pub dropped_verbose: u64,
    pub dropped_debug: u64,
    pub dropped_log: u64,
    pub dropped_warn: u64,
    pub dropped_error: u64,
    pub critical_backpressure_count: u64,
    pub output_errors: u64,
    pub closed: bool,
}

pub(super) struct SinkMetrics {
    max_queue_depth: AtomicUsize,
    dequeued_records: AtomicU64,
    written_records: AtomicU64,
    written_bytes: AtomicU64,
    flush_batches: AtomicU64,
    max_queue_age_nanos: AtomicU64,
    dropped_verbose: AtomicU64,
    dropped_debug: AtomicU64,
    dropped_log: AtomicU64,
    dropped_warn: AtomicU64,
    dropped_error: AtomicU64,
    critical_backpressure_count: AtomicU64,
    output_errors: AtomicU64,
    closed: AtomicBool,
}

impl SinkMetrics {
    pub(super) fn new() -> Self {
        Self {
            max_queue_depth: AtomicUsize::new(0),
            dequeued_records: AtomicU64::new(0),
            written_records: AtomicU64::new(0),
            written_bytes: AtomicU64::new(0),
            flush_batches: AtomicU64::new(0),
            max_queue_age_nanos: AtomicU64::new(0),
            dropped_verbose: AtomicU64::new(0),
            dropped_debug: AtomicU64::new(0),
            dropped_log: AtomicU64::new(0),
            dropped_warn: AtomicU64::new(0),
            dropped_error: AtomicU64::new(0),
            critical_backpressure_count: AtomicU64::new(0),
            output_errors: AtomicU64::new(0),
            closed: AtomicBool::new(false),
        }
    }

    pub(super) fn observe_queue_depth(&self, depth: usize) {
        self.max_queue_depth.fetch_max(depth, Ordering::Relaxed);
    }

    pub(super) fn record_dequeued(&self, enqueued_at: Instant) {
        self.dequeued_records.fetch_add(1, Ordering::Relaxed);
        let age = duration_nanos_u64(enqueued_at.elapsed());
        self.max_queue_age_nanos.fetch_max(age, Ordering::Relaxed);
    }

    pub(super) fn record_drop(&self, level: DiagnosticLogLevel) {
        let counter = match level {
            DiagnosticLogLevel::Verbose => &self.dropped_verbose,
            DiagnosticLogLevel::Debug => &self.dropped_debug,
            DiagnosticLogLevel::Log => &self.dropped_log,
            DiagnosticLogLevel::Warn => &self.dropped_warn,
            DiagnosticLogLevel::Error => &self.dropped_error,
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_critical_backpressure(&self) {
        self.critical_backpressure_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_batch(&self, records: usize, bytes: usize, output_succeeded: bool) {
        self.flush_batches.fetch_add(1, Ordering::Relaxed);
        if !output_succeeded {
            return;
        }
        self.written_records
            .fetch_add(records as u64, Ordering::Relaxed);
        self.written_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(super) fn record_output_error(&self) {
        self.output_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn mark_closed(&self) {
        self.closed.store(true, Ordering::Release);
    }

    pub(super) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    pub(super) fn outputs_succeeded(&self) -> bool {
        self.output_errors.load(Ordering::Acquire) == 0
    }

    pub(super) fn snapshot(&self, queue_depth: usize) -> DiagnosticLogSinkSnapshot {
        DiagnosticLogSinkSnapshot {
            queue_depth,
            max_queue_depth: self.max_queue_depth.load(Ordering::Relaxed),
            dequeued_records: self.dequeued_records.load(Ordering::Relaxed),
            written_records: self.written_records.load(Ordering::Relaxed),
            written_bytes: self.written_bytes.load(Ordering::Relaxed),
            flush_batches: self.flush_batches.load(Ordering::Relaxed),
            max_queue_age: Duration::from_nanos(self.max_queue_age_nanos.load(Ordering::Relaxed)),
            dropped_verbose: self.dropped_verbose.load(Ordering::Relaxed),
            dropped_debug: self.dropped_debug.load(Ordering::Relaxed),
            dropped_log: self.dropped_log.load(Ordering::Relaxed),
            dropped_warn: self.dropped_warn.load(Ordering::Relaxed),
            dropped_error: self.dropped_error.load(Ordering::Relaxed),
            critical_backpressure_count: self.critical_backpressure_count.load(Ordering::Relaxed),
            output_errors: self.output_errors.load(Ordering::Relaxed),
            closed: self.closed.load(Ordering::Acquire),
        }
    }
}

fn duration_nanos_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
