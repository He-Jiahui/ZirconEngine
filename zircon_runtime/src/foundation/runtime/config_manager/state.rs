use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::core::framework::foundation::ConfigPersistenceReport;

const MAX_FLUSH_LATENCY_SAMPLES: usize = 64;

#[derive(Debug, Default)]
pub(super) struct ConfigPersistenceState {
    pub(super) dirty_generation: u64,
    pub(super) persisted_generation: u64,
    pub(super) attempted_generation: u64,
    pub(super) work_requested: bool,
    pub(super) force_flush: bool,
    pub(super) attempt_in_flight: bool,
    pub(super) shutdown_requested: bool,
    pub(super) worker_exited: bool,
    pub(super) last_dirty_at: Option<Instant>,
    flush_attempts: u64,
    successful_writes: u64,
    failed_writes: u64,
    serialized_bytes: u64,
    peak_pending_flushes: u64,
    flush_latencies_ns: VecDeque<u64>,
    pub(super) last_error: Option<String>,
}

impl ConfigPersistenceState {
    pub(super) fn request_persistence(&mut self, changed: bool) -> bool {
        let was_requested = self.work_requested;
        if changed {
            self.dirty_generation = self.dirty_generation.saturating_add(1);
        }
        if self.dirty_generation <= self.persisted_generation {
            return false;
        }

        if changed || !was_requested {
            self.last_dirty_at = Some(Instant::now());
        }
        self.work_requested = true;
        self.peak_pending_flushes = self.peak_pending_flushes.max(1);
        changed || !was_requested
    }

    pub(super) fn request_force_flush(&mut self) -> u64 {
        let target_generation = self.dirty_generation;
        if target_generation > self.persisted_generation {
            self.work_requested = true;
            self.force_flush = true;
            self.peak_pending_flushes = self.peak_pending_flushes.max(1);
        }
        target_generation
    }

    pub(super) fn begin_attempt(&mut self) -> u64 {
        self.work_requested = false;
        self.force_flush = false;
        self.attempt_in_flight = true;
        self.dirty_generation
    }

    pub(super) fn complete_attempt(
        &mut self,
        target_generation: u64,
        serialized_bytes: usize,
        elapsed: Duration,
        error: Option<String>,
    ) {
        self.attempt_in_flight = false;
        self.attempted_generation = self.attempted_generation.max(target_generation);
        self.flush_attempts = self.flush_attempts.saturating_add(1);
        self.serialized_bytes = self
            .serialized_bytes
            .saturating_add(serialized_bytes as u64);
        self.record_latency(elapsed);

        if let Some(error) = error {
            self.failed_writes = self.failed_writes.saturating_add(1);
            self.last_error = Some(error);
            return;
        }

        self.successful_writes = self.successful_writes.saturating_add(1);
        self.persisted_generation = self.persisted_generation.max(target_generation);
        self.last_error = None;
        if self.dirty_generation > self.persisted_generation {
            self.work_requested = true;
        } else {
            self.work_requested = false;
            self.force_flush = false;
        }
    }

    pub(super) fn report(&self) -> ConfigPersistenceReport {
        ConfigPersistenceReport {
            dirty_generation: self.dirty_generation,
            persisted_generation: self.persisted_generation,
            pending_flushes: u64::from(self.dirty_generation > self.persisted_generation),
            peak_pending_flushes: self.peak_pending_flushes,
            flush_attempts: self.flush_attempts,
            successful_writes: self.successful_writes,
            failed_writes: self.failed_writes,
            serialized_bytes: self.serialized_bytes,
            flush_p95_ms: percentile_ms(&self.flush_latencies_ns, 95),
            max_flush_ms: self
                .flush_latencies_ns
                .iter()
                .copied()
                .max()
                .map_or(0.0, duration_ms),
            last_error: self.last_error.clone(),
        }
    }

    fn record_latency(&mut self, elapsed: Duration) {
        if self.flush_latencies_ns.len() == MAX_FLUSH_LATENCY_SAMPLES {
            self.flush_latencies_ns.pop_front();
        }
        self.flush_latencies_ns.push_back(duration_ns(elapsed));
    }
}

fn percentile_ms(samples: &VecDeque<u64>, percentile: usize) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut ordered = samples.iter().copied().collect::<Vec<_>>();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100).saturating_sub(1);
    duration_ms(ordered[rank])
}

fn duration_ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

fn duration_ms(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::ConfigPersistenceState;

    #[test]
    fn unchanged_value_does_not_postpone_an_already_requested_generation() {
        let mut state = ConfigPersistenceState::default();
        assert!(state.request_persistence(true));
        let first_dirty_at = state.last_dirty_at;

        assert!(!state.request_persistence(false));
        assert_eq!(state.last_dirty_at, first_dirty_at);
        assert_eq!(state.dirty_generation, 1);
    }
}
