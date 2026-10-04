use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::framework::events::{EventBusDiagnosticsMode, EventBusDiagnosticsSnapshot};

/// `delivered` 统计成功入队次数（含替换旧项），不表示消费者已经读取事件。
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
    pub(super) fn record_evicted(&self, queued_at: Option<Instant>) {
        if !self.enabled {
            return;
        }
        self.dropped.fetch_add(1, Ordering::Relaxed);
        self.record_dequeued_depth();
        self.record_dequeued_age(queued_at.map(|queued_at| queued_at.elapsed()));
    }

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

    // 批量排空一次性扣减队列深度；只有记录过入队时刻的项目才贡献年龄样本。
    pub(super) fn record_drained(&self, queued_at: impl IntoIterator<Item = Option<Instant>>) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        let mut drained = 0_u64;
        let mut age_samples = 0_u64;
        let mut total_age_ns = 0_u64;
        let mut max_age_ns = 0_u64;
        for queued_at in queued_at {
            drained = drained.saturating_add(1);
            let Some(queued_at) = queued_at else {
                continue;
            };
            let age_ns = duration_ns(now.saturating_duration_since(queued_at));
            age_samples = age_samples.saturating_add(1);
            total_age_ns = total_age_ns.saturating_add(age_ns);
            max_age_ns = max_age_ns.max(age_ns);
        }
        if drained == 0 {
            return;
        }
        decrement_saturating_by(&self.queued, drained);
        if age_samples == 0 {
            return;
        }
        self.queue_age_samples
            .fetch_add(age_samples, Ordering::Relaxed);
        self.total_queue_age_ns
            .fetch_add(total_age_ns, Ordering::Relaxed);
        update_max(&self.max_queue_age_ns, max_age_ns);
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
    decrement_saturating_by(value, 1);
}

fn decrement_saturating_by(value: &AtomicU64, amount: u64) {
    let _ = value.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        Some(current.saturating_sub(amount))
    });
}

fn update_max(target: &AtomicU64, candidate: u64) {
    let _ = target.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        (candidate > current).then_some(candidate)
    });
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;

#[cfg(test)]
#[path = "diagnostics/tests/optimization_batch_js_runtime658_tests.rs"]
mod optimization_batch_js_runtime658_tests;
