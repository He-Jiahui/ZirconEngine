use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::framework::events::{EventBusDiagnosticsMode, EventBusDiagnosticsSnapshot};

pub(super) struct EventBusDiagnosticsState {
    enabled: bool,
    routine_timing_sample_interval: u64,
    published: AtomicU64,
    delivered: AtomicU64,
    dropped: AtomicU64,
    disconnected: AtomicU64,
    queued: AtomicU64,
    peak_queued: AtomicU64,
    waiting_receivers: AtomicU64,
    waiting_publishers: AtomicU64,
    queue_age_samples: AtomicU64,
    total_queue_age_ns: AtomicU64,
    max_queue_age_ns: AtomicU64,
    publish_samples: AtomicU64,
    total_publish_ns: AtomicU64,
    max_publish_ns: AtomicU64,
    delivery_lock_wait_samples: AtomicU64,
    total_delivery_lock_wait_ns: AtomicU64,
    max_delivery_lock_wait_ns: AtomicU64,
}

impl EventBusDiagnosticsState {
    pub(super) fn new(mode: EventBusDiagnosticsMode) -> Self {
        let (enabled, routine_timing_sample_interval) = match mode {
            EventBusDiagnosticsMode::Enabled => (true, 1),
            EventBusDiagnosticsMode::Sampled { every } => (true, every.get()),
            EventBusDiagnosticsMode::Disabled => (false, 0),
        };
        Self {
            enabled,
            routine_timing_sample_interval,
            published: AtomicU64::new(0),
            delivered: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            disconnected: AtomicU64::new(0),
            queued: AtomicU64::new(0),
            peak_queued: AtomicU64::new(0),
            waiting_receivers: AtomicU64::new(0),
            waiting_publishers: AtomicU64::new(0),
            queue_age_samples: AtomicU64::new(0),
            total_queue_age_ns: AtomicU64::new(0),
            max_queue_age_ns: AtomicU64::new(0),
            publish_samples: AtomicU64::new(0),
            total_publish_ns: AtomicU64::new(0),
            max_publish_ns: AtomicU64::new(0),
            delivery_lock_wait_samples: AtomicU64::new(0),
            total_delivery_lock_wait_ns: AtomicU64::new(0),
            max_delivery_lock_wait_ns: AtomicU64::new(0),
        }
    }

    pub(super) fn capture_contention_time(&self) -> Option<Instant> {
        self.enabled.then(Instant::now)
    }

    pub(super) fn record_published_and_capture_time(&self) -> Option<Instant> {
        if !self.enabled {
            return None;
        }
        let sample_index = self.published.fetch_add(1, Ordering::Relaxed);
        self.capture_routine_time(sample_index)
    }

    pub(super) fn record_enqueued_and_capture_time(&self) -> Option<Instant> {
        if !self.enabled {
            return None;
        }
        let queued = self.queued.fetch_add(1, Ordering::AcqRel) + 1;
        let sample_index = self.delivered.fetch_add(1, Ordering::Relaxed);
        update_max(&self.peak_queued, queued);
        self.capture_routine_time(sample_index)
    }

    pub(super) fn record_dequeued(&self, queued_at: Option<Instant>) {
        self.record_dequeued_depth();
        self.record_dequeued_age(queued_at.map(|queued_at| queued_at.elapsed()));
    }

    pub(super) fn record_dequeued_depth(&self) {
        if !self.enabled {
            return;
        }
        decrement_saturating(&self.queued);
    }

    pub(super) fn record_dequeued_age(&self, queue_age: Option<Duration>) {
        if !self.enabled {
            return;
        }
        let Some(queue_age) = queue_age else {
            return;
        };
        self.record_queue_age(queue_age);
    }

    pub(super) fn record_replaced_and_capture_time(
        &self,
        queued_at: Option<Instant>,
    ) -> Option<Instant> {
        if !self.enabled {
            return None;
        }
        self.dropped.fetch_add(1, Ordering::Relaxed);
        self.record_dequeued_age(queued_at.map(|queued_at| queued_at.elapsed()));
        let sample_index = self.delivered.fetch_add(1, Ordering::Relaxed);
        self.capture_routine_time(sample_index)
    }

    pub(super) fn record_receiver_waiting(&self) {
        if !self.enabled {
            return;
        }
        self.waiting_receivers.fetch_add(1, Ordering::AcqRel);
    }

    pub(super) fn record_receiver_resumed(&self) {
        if !self.enabled {
            return;
        }
        decrement_saturating(&self.waiting_receivers);
    }

    pub(super) fn record_disconnected(&self) {
        if !self.enabled {
            return;
        }
        self.disconnected.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_publisher_waiting(&self) {
        if !self.enabled {
            return;
        }
        self.waiting_publishers.fetch_add(1, Ordering::AcqRel);
    }

    pub(super) fn record_publisher_resumed(&self, wait_started: Option<Instant>) {
        if !self.enabled {
            return;
        }
        decrement_saturating(&self.waiting_publishers);
        let Some(wait_started) = wait_started else {
            return;
        };
        let nanos = duration_ns(wait_started.elapsed());
        self.delivery_lock_wait_samples
            .fetch_add(1, Ordering::Relaxed);
        self.total_delivery_lock_wait_ns
            .fetch_add(nanos, Ordering::Relaxed);
        update_max(&self.max_delivery_lock_wait_ns, nanos);
    }

    pub(super) fn record_publish_duration(&self, started: Option<Instant>) {
        if !self.enabled {
            return;
        }
        let Some(started) = started else {
            return;
        };
        let elapsed = started.elapsed();
        let nanos = duration_ns(elapsed);
        self.publish_samples.fetch_add(1, Ordering::Relaxed);
        self.total_publish_ns.fetch_add(nanos, Ordering::Relaxed);
        update_max(&self.max_publish_ns, nanos);
    }

    pub(super) fn snapshot(
        &self,
        topics: usize,
        subscribers: usize,
    ) -> EventBusDiagnosticsSnapshot {
        if !self.enabled {
            return EventBusDiagnosticsSnapshot {
                enabled: false,
                topics: topics as u64,
                subscribers: subscribers as u64,
                ..EventBusDiagnosticsSnapshot::default()
            };
        }
        let queued = self.queued.load(Ordering::Acquire);
        let peak_queued = self.peak_queued.load(Ordering::Acquire).max(queued);
        EventBusDiagnosticsSnapshot {
            enabled: self.enabled,
            routine_timing_sample_interval: self.routine_timing_sample_interval,
            topics: topics as u64,
            subscribers: subscribers as u64,
            published: self.published.load(Ordering::Acquire),
            delivered: self.delivered.load(Ordering::Acquire),
            dropped: self.dropped.load(Ordering::Acquire),
            disconnected: self.disconnected.load(Ordering::Acquire),
            queued,
            peak_queued,
            waiting_receivers: self.waiting_receivers.load(Ordering::Acquire),
            waiting_publishers: self.waiting_publishers.load(Ordering::Acquire),
            queue_age_samples: self.queue_age_samples.load(Ordering::Acquire),
            total_queue_age_ms: duration_ms(self.total_queue_age_ns.load(Ordering::Acquire)),
            max_queue_age_ms: duration_ms(self.max_queue_age_ns.load(Ordering::Acquire)),
            publish_samples: self.publish_samples.load(Ordering::Acquire),
            total_publish_ms: duration_ms(self.total_publish_ns.load(Ordering::Acquire)),
            max_publish_ms: duration_ms(self.max_publish_ns.load(Ordering::Acquire)),
            delivery_lock_wait_samples: self.delivery_lock_wait_samples.load(Ordering::Acquire),
            total_delivery_lock_wait_ms: duration_ms(
                self.total_delivery_lock_wait_ns.load(Ordering::Acquire),
            ),
            max_delivery_lock_wait_ms: duration_ms(
                self.max_delivery_lock_wait_ns.load(Ordering::Acquire),
            ),
        }
    }

    fn record_queue_age(&self, queue_age: Duration) {
        let nanos = duration_ns(queue_age);
        self.queue_age_samples.fetch_add(1, Ordering::Relaxed);
        self.total_queue_age_ns.fetch_add(nanos, Ordering::Relaxed);
        update_max(&self.max_queue_age_ns, nanos);
    }

    fn capture_routine_time(&self, sample_index: u64) -> Option<Instant> {
        sample_due(sample_index, self.routine_timing_sample_interval).then(Instant::now)
    }
}

fn sample_due(sample_index: u64, interval: u64) -> bool {
    if interval == 0 {
        return false;
    }
    if interval.is_power_of_two() {
        sample_index & (interval - 1) == 0
    } else {
        sample_index % interval == 0
    }
}

fn duration_ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

fn duration_ms(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn decrement_saturating(value: &AtomicU64) {
    let _ = value.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        Some(current.saturating_sub(1))
    });
}

fn update_max(target: &AtomicU64, candidate: u64) {
    let _ = target.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        (candidate > current).then_some(candidate)
    });
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::sample_due;

    #[test]
    fn optimization_batch_20260831ez_runtime565_power_of_two_sampling_matches_modulo_semantics() {
        for interval in [1, 2, 4, 8, 64, 128] {
            for sample_index in 0..512 {
                assert_eq!(
                    sample_due(sample_index, interval),
                    sample_index % interval == 0,
                    "interval={interval} sample_index={sample_index}"
                );
            }
        }
    }

    #[test]
    fn optimization_batch_20260831ez_runtime565_non_power_of_two_sampling_keeps_modulo_semantics() {
        for interval in [3, 5, 7, 63, 65] {
            for sample_index in 0..512 {
                assert_eq!(
                    sample_due(sample_index, interval),
                    sample_index % interval == 0,
                    "interval={interval} sample_index={sample_index}"
                );
            }
        }
        assert!(!sample_due(0, 0));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260831ez_runtime565_event_sampling_mask_p95() {
        const SAMPLE_PAIRS: usize = 13;
        const ITERATIONS: u64 = 20_000_000;
        const INTERVAL: u64 = 64;
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, INTERVAL, ITERATIONS));
                optimized.push(measure(true, INTERVAL, ITERATIONS));
            } else {
                optimized.push(measure(true, INTERVAL, ITERATIONS));
                legacy.push(measure(false, INTERVAL, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME565_EVENT_SAMPLING_MASK_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} interval={INTERVAL} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50));
    }

    fn measure(optimized: bool, interval: u64, iterations: u64) -> u128 {
        let started = Instant::now();
        let mut hits = 0_u64;
        let interval = black_box(interval);
        for sample_index in 0..iterations {
            hits += u64::from(if optimized {
                sample_due(sample_index, interval)
            } else {
                sample_index % interval == 0
            });
        }
        black_box(hits);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
