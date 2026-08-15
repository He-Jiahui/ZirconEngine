use std::time::Duration;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::CoreHandle;

pub const NATIVE_SYSTEM_CONFLICT_COUNT_DIAGNOSTIC: &str = "scene.ecs.native_system.conflict_count";
pub const NATIVE_SYSTEM_READY_DELAY_MS_DIAGNOSTIC: &str = "scene.ecs.native_system.ready_delay_ms";
pub const NATIVE_SYSTEM_WORKER_UTILIZATION_DIAGNOSTIC: &str =
    "scene.ecs.native_system.worker_utilization";
pub const NATIVE_SYSTEM_CALLBACK_P95_MS_DIAGNOSTIC: &str =
    "scene.ecs.native_system.callback_p95_ms";
pub const NATIVE_SYSTEM_WORKER_BATCH_COUNT_DIAGNOSTIC: &str =
    "scene.ecs.native_system.worker_batch_count";
pub const NATIVE_SYSTEM_CALLBACK_COUNT_DIAGNOSTIC: &str = "scene.ecs.native_system.callback_count";
pub const NATIVE_SYSTEM_CONSERVATIVE_WORLD_WRITER_COUNT_DIAGNOSTIC: &str =
    "scene.ecs.native_system.conservative_world_writer_count";
pub const NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_COUNT_DIAGNOSTIC: &str =
    "scene.ecs.native_system.temporary_control_buffer_count";
pub const NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_BYTES_DIAGNOSTIC: &str =
    "scene.ecs.native_system.temporary_control_buffer_bytes";

const CALLBACK_LATENCY_BUCKET_COUNT: usize = 65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSystemScheduleDiagnostics {
    conflict_count: u64,
    worker_batch_count: u64,
    callback_count: u64,
    conservative_world_writer_count: u64,
    temporary_control_buffer_count: u64,
    temporary_control_buffer_bytes: u64,
    worker_callback_count: u64,
    ready_delay_ns: u64,
    worker_busy_ns: u64,
    worker_capacity_ns: u64,
    callback_latency_buckets: [u64; CALLBACK_LATENCY_BUCKET_COUNT],
}

impl Default for NativeSystemScheduleDiagnostics {
    fn default() -> Self {
        Self {
            conflict_count: 0,
            worker_batch_count: 0,
            callback_count: 0,
            conservative_world_writer_count: 0,
            temporary_control_buffer_count: 0,
            temporary_control_buffer_bytes: 0,
            worker_callback_count: 0,
            ready_delay_ns: 0,
            worker_busy_ns: 0,
            worker_capacity_ns: 0,
            callback_latency_buckets: [0; CALLBACK_LATENCY_BUCKET_COUNT],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeSystemCallbackTiming {
    pub(crate) ready_delay: Duration,
    pub(crate) callback: Duration,
}

impl NativeSystemScheduleDiagnostics {
    pub(crate) fn record_conflicts(&mut self, count: usize) {
        self.conflict_count = self.conflict_count.saturating_add(count as u64);
    }

    pub(crate) fn record_main_callback(
        &mut self,
        callback: Duration,
        conservative_world_writer: bool,
    ) {
        self.callback_count = self.callback_count.saturating_add(1);
        if conservative_world_writer {
            self.conservative_world_writer_count =
                self.conservative_world_writer_count.saturating_add(1);
        }
        self.record_callback_latency(callback);
    }

    pub(crate) fn record_worker_batch(
        &mut self,
        timings: &[NativeSystemCallbackTiming],
        elapsed: Duration,
        scheduler_parallelism: usize,
        temporary_control_buffer_count: usize,
        temporary_control_buffer_bytes: usize,
    ) {
        if timings.is_empty() {
            return;
        }
        self.worker_batch_count = self.worker_batch_count.saturating_add(1);
        self.callback_count = self.callback_count.saturating_add(timings.len() as u64);
        self.worker_callback_count = self
            .worker_callback_count
            .saturating_add(timings.len() as u64);
        self.temporary_control_buffer_count = self
            .temporary_control_buffer_count
            .saturating_add(temporary_control_buffer_count as u64);
        self.temporary_control_buffer_bytes = self
            .temporary_control_buffer_bytes
            .saturating_add(temporary_control_buffer_bytes as u64);
        let active_workers = timings.len().min(scheduler_parallelism.max(1));
        self.worker_capacity_ns = self
            .worker_capacity_ns
            .saturating_add(duration_ns(elapsed).saturating_mul(active_workers as u64));
        for timing in timings {
            self.ready_delay_ns = self
                .ready_delay_ns
                .saturating_add(duration_ns(timing.ready_delay));
            self.worker_busy_ns = self
                .worker_busy_ns
                .saturating_add(duration_ns(timing.callback));
            self.record_callback_latency(timing.callback);
        }
    }

    pub fn conflict_count(&self) -> u64 {
        self.conflict_count
    }

    pub fn worker_batch_count(&self) -> u64 {
        self.worker_batch_count
    }

    pub fn callback_count(&self) -> u64 {
        self.callback_count
    }

    pub fn conservative_world_writer_count(&self) -> u64 {
        self.conservative_world_writer_count
    }

    pub fn temporary_control_buffer_count(&self) -> u64 {
        self.temporary_control_buffer_count
    }

    pub fn temporary_control_buffer_bytes(&self) -> u64 {
        self.temporary_control_buffer_bytes
    }

    pub fn ready_delay_ms(&self) -> f64 {
        if self.worker_callback_count == 0 {
            return 0.0;
        }
        self.ready_delay_ns as f64 / self.worker_callback_count as f64 / 1_000_000.0
    }

    pub fn worker_utilization(&self) -> f64 {
        if self.worker_capacity_ns == 0 {
            return 0.0;
        }
        (self.worker_busy_ns as f64 / self.worker_capacity_ns as f64).clamp(0.0, 1.0)
    }

    pub fn callback_p95_ms(&self) -> f64 {
        percentile_bucket_upper_bound_ns(&self.callback_latency_buckets, 95) as f64 / 1_000_000.0
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        for (path, value, unit) in self.diagnostic_values() {
            store.record(
                path,
                frame_index,
                value,
                Some(unit),
                ["ecs", "native_system", "schedule"],
            );
        }
    }

    pub fn publish(&self, core: &CoreHandle, frame_index: u64) {
        for (path, value, unit) in self.diagnostic_values() {
            core.record_diagnostic(
                path,
                frame_index,
                value,
                Some(unit),
                ["ecs", "native_system", "schedule"],
            );
        }
    }

    fn record_callback_latency(&mut self, callback: Duration) {
        let bucket = duration_ns(callback)
            .checked_ilog2()
            .map(|index| index as usize + 1)
            .unwrap_or(0)
            .min(CALLBACK_LATENCY_BUCKET_COUNT - 1);
        self.callback_latency_buckets[bucket] =
            self.callback_latency_buckets[bucket].saturating_add(1);
    }

    fn diagnostic_values(&self) -> [(&'static str, f64, &'static str); 9] {
        [
            (
                NATIVE_SYSTEM_CONFLICT_COUNT_DIAGNOSTIC,
                self.conflict_count as f64,
                "conflict",
            ),
            (
                NATIVE_SYSTEM_READY_DELAY_MS_DIAGNOSTIC,
                self.ready_delay_ms(),
                "ms",
            ),
            (
                NATIVE_SYSTEM_WORKER_UTILIZATION_DIAGNOSTIC,
                self.worker_utilization(),
                "ratio",
            ),
            (
                NATIVE_SYSTEM_CALLBACK_P95_MS_DIAGNOSTIC,
                self.callback_p95_ms(),
                "ms",
            ),
            (
                NATIVE_SYSTEM_WORKER_BATCH_COUNT_DIAGNOSTIC,
                self.worker_batch_count as f64,
                "batch",
            ),
            (
                NATIVE_SYSTEM_CALLBACK_COUNT_DIAGNOSTIC,
                self.callback_count as f64,
                "callback",
            ),
            (
                NATIVE_SYSTEM_CONSERVATIVE_WORLD_WRITER_COUNT_DIAGNOSTIC,
                self.conservative_world_writer_count as f64,
                "callback",
            ),
            (
                NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_COUNT_DIAGNOSTIC,
                self.temporary_control_buffer_count as f64,
                "buffer",
            ),
            (
                NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_BYTES_DIAGNOSTIC,
                self.temporary_control_buffer_bytes as f64,
                "byte",
            ),
        ]
    }
}

fn duration_ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u64::MAX as u128) as u64
}

fn percentile_bucket_upper_bound_ns(
    buckets: &[u64; CALLBACK_LATENCY_BUCKET_COUNT],
    percentile: u64,
) -> u64 {
    let sample_count = buckets
        .iter()
        .fold(0_u64, |total, count| total.saturating_add(*count));
    if sample_count == 0 {
        return 0;
    }
    let target = sample_count.saturating_mul(percentile).saturating_add(99) / 100;
    let mut observed = 0_u64;
    for (bucket, count) in buckets.iter().copied().enumerate() {
        observed = observed.saturating_add(count);
        if observed >= target {
            return match bucket {
                0 => 0,
                64 => u64::MAX,
                _ => (1_u64 << bucket) - 1,
            };
        }
    }
    u64::MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_system_schedule_diagnostics_record_conflicts_latency_and_utilization() {
        let mut diagnostics = NativeSystemScheduleDiagnostics::default();
        diagnostics.record_conflicts(3);
        diagnostics.record_main_callback(Duration::from_micros(10), true);
        diagnostics.record_worker_batch(
            &[
                NativeSystemCallbackTiming {
                    ready_delay: Duration::from_micros(4),
                    callback: Duration::from_micros(20),
                },
                NativeSystemCallbackTiming {
                    ready_delay: Duration::from_micros(8),
                    callback: Duration::from_micros(30),
                },
            ],
            Duration::from_micros(40),
            2,
            3,
            384,
        );

        assert_eq!(diagnostics.conflict_count(), 3);
        assert_eq!(diagnostics.worker_batch_count(), 1);
        assert_eq!(diagnostics.callback_count(), 3);
        assert_eq!(diagnostics.conservative_world_writer_count(), 1);
        assert_eq!(diagnostics.temporary_control_buffer_count(), 3);
        assert_eq!(diagnostics.temporary_control_buffer_bytes(), 384);
        assert!((diagnostics.ready_delay_ms() - 0.006).abs() < f64::EPSILON);
        assert_eq!(diagnostics.worker_utilization(), 0.625);
        assert!(diagnostics.callback_p95_ms() >= 0.03);

        let mut store = DiagnosticStore::default();
        diagnostics.record_diagnostics(&mut store, 7);
        let snapshot = store.snapshot();
        assert_eq!(snapshot.series.len(), 9);
        assert!(snapshot.series.iter().any(|series| {
            series.path.as_str() == NATIVE_SYSTEM_CONFLICT_COUNT_DIAGNOSTIC
                && series.current == Some(3.0)
        }));
        assert!(snapshot.series.iter().any(|series| {
            series.path.as_str() == NATIVE_SYSTEM_CALLBACK_P95_MS_DIAGNOSTIC
                && series.current.is_some_and(|value| value >= 0.03)
        }));
        assert!(snapshot.series.iter().any(|series| {
            series.path.as_str() == NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_COUNT_DIAGNOSTIC
                && series.current == Some(3.0)
        }));
        assert!(snapshot.series.iter().any(|series| {
            series.path.as_str() == NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_BYTES_DIAGNOSTIC
                && series.current == Some(384.0)
        }));
    }
}
