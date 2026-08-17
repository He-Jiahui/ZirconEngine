//! Lock-free counters and snapshots for foreign-output activity.

use std::array;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::kind::{RuntimeForeignOutputKind, RUNTIME_FOREIGN_OUTPUT_KIND_COUNT};

#[derive(Default)]
pub(super) struct RuntimeForeignOutputCounters {
    accepted_payloads: AtomicU64,
    accepted_bytes: AtomicU64,
    rejected_payloads: AtomicU64,
    rejected_bytes: AtomicU64,
    call_failures: AtomicU64,
    blocked_calls: AtomicU64,
    total_decode_nanoseconds: AtomicU64,
    max_decode_nanoseconds: AtomicU64,
}

impl RuntimeForeignOutputCounters {
    pub(super) fn snapshot(&self) -> RuntimeForeignOutputMetrics {
        RuntimeForeignOutputMetrics {
            accepted_payloads: self.accepted_payloads.load(Ordering::Relaxed),
            accepted_bytes: self.accepted_bytes.load(Ordering::Relaxed),
            rejected_payloads: self.rejected_payloads.load(Ordering::Relaxed),
            rejected_bytes: self.rejected_bytes.load(Ordering::Relaxed),
            call_failures: self.call_failures.load(Ordering::Relaxed),
            blocked_calls: self.blocked_calls.load(Ordering::Relaxed),
            total_decode_nanoseconds: self.total_decode_nanoseconds.load(Ordering::Relaxed),
            max_decode_nanoseconds: self.max_decode_nanoseconds.load(Ordering::Relaxed),
        }
    }

    pub(super) fn record_accepted(&self, encoded_len: usize, decode_time: Duration) {
        self.accepted_payloads.fetch_add(1, Ordering::Relaxed);
        self.accepted_bytes
            .fetch_add(usize_to_u64(encoded_len), Ordering::Relaxed);
        self.record_decode_time(decode_time);
    }

    pub(super) fn record_rejected(&self, encoded_len: usize, decode_time: Duration) {
        self.rejected_payloads.fetch_add(1, Ordering::Relaxed);
        self.rejected_bytes
            .fetch_add(usize_to_u64(encoded_len), Ordering::Relaxed);
        self.record_decode_time(decode_time);
    }

    pub(super) fn record_call_failure(&self) {
        self.call_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_blocked_call(&self) {
        self.blocked_calls.fetch_add(1, Ordering::Relaxed);
    }

    fn record_decode_time(&self, decode_time: Duration) {
        if decode_time.is_zero() {
            return;
        }
        let decode_nanoseconds = duration_to_u64_nanoseconds(decode_time).max(1);
        self.total_decode_nanoseconds
            .fetch_add(decode_nanoseconds, Ordering::Relaxed);
        self.max_decode_nanoseconds
            .fetch_max(decode_nanoseconds, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeForeignOutputMetrics {
    pub accepted_payloads: u64,
    pub accepted_bytes: u64,
    pub rejected_payloads: u64,
    pub rejected_bytes: u64,
    pub call_failures: u64,
    pub blocked_calls: u64,
    pub total_decode_nanoseconds: u64,
    pub max_decode_nanoseconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeForeignOutputMetricsSnapshot {
    pub protocol_failed: bool,
    pub protocol_failures: u64,
    pub blocked_session_calls: u64,
    pub(super) by_kind: [RuntimeForeignOutputMetrics; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT],
}

impl RuntimeForeignOutputMetricsSnapshot {
    pub const fn for_kind(self, kind: RuntimeForeignOutputKind) -> RuntimeForeignOutputMetrics {
        self.by_kind[kind.index()]
    }

    pub(super) fn has_activity(self) -> bool {
        self.protocol_failures > 0
            || self.blocked_session_calls > 0
            || self.by_kind.iter().any(|metrics| {
                metrics.accepted_payloads > 0
                    || metrics.rejected_payloads > 0
                    || metrics.call_failures > 0
                    || metrics.blocked_calls > 0
            })
    }
}

pub(super) fn empty_counters() -> [RuntimeForeignOutputCounters; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT]
{
    array::from_fn(|_| RuntimeForeignOutputCounters::default())
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn duration_to_u64_nanoseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
