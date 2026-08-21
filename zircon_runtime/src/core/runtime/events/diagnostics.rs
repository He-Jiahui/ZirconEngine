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
        if !self.enabled {
            return;
        }
        decrement_saturating(&self.queued);
        self.record_queue_age(queued_at);
    }

    pub(super) fn record_replaced_and_capture_time(
        &self,
        queued_at: Option<Instant>,
    ) -> Option<Instant> {
        if !self.enabled {
            return None;
        }
        self.dropped.fetch_add(1, Ordering::Relaxed);
        self.record_queue_age(queued_at);
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

    fn record_queue_age(&self, queued_at: Option<Instant>) {
        let Some(queued_at) = queued_at else {
            return;
        };
        let age = queued_at.elapsed();
        let nanos = duration_ns(age);
        self.queue_age_samples.fetch_add(1, Ordering::Relaxed);
        self.total_queue_age_ns.fetch_add(nanos, Ordering::Relaxed);
        update_max(&self.max_queue_age_ns, nanos);
    }

    fn capture_routine_time(&self, sample_index: u64) -> Option<Instant> {
        (sample_index % self.routine_timing_sample_interval == 0).then(Instant::now)
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
