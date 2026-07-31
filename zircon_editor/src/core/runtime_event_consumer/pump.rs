use std::time::Duration;

const DEFAULT_MAX_EVENTS: usize = 256;
const DEFAULT_MAX_EVENTS_PER_CONSUMER: usize = 64;
const DEFAULT_MAX_ELAPSED: Duration = Duration::from_millis(4);
const DEFAULT_SLOW_CALLBACK_THRESHOLD: Duration = Duration::from_millis(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorRuntimeEventPumpBudget {
    max_events: usize,
    max_events_per_consumer: usize,
    max_elapsed: Duration,
    slow_callback_threshold: Duration,
}

impl EditorRuntimeEventPumpBudget {
    pub const fn new(
        max_events: usize,
        max_events_per_consumer: usize,
        max_elapsed: Duration,
        slow_callback_threshold: Duration,
    ) -> Self {
        Self {
            max_events,
            max_events_per_consumer,
            max_elapsed,
            slow_callback_threshold,
        }
    }

    pub const fn max_events(self) -> usize {
        self.max_events
    }

    pub const fn max_events_per_consumer(self) -> usize {
        self.max_events_per_consumer
    }

    pub const fn max_elapsed(self) -> Duration {
        self.max_elapsed
    }

    pub const fn slow_callback_threshold(self) -> Duration {
        self.slow_callback_threshold
    }
}

impl Default for EditorRuntimeEventPumpBudget {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_EVENTS,
            DEFAULT_MAX_EVENTS_PER_CONSUMER,
            DEFAULT_MAX_ELAPSED,
            DEFAULT_SLOW_CALLBACK_THRESHOLD,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorRuntimeEventPumpReport {
    applied: usize,
    drained: usize,
    drained_encoded_bytes: usize,
    runtime_drain_elapsed: Duration,
    runtime_drain_p95: Duration,
    decode_elapsed: Duration,
    decode_p95: Duration,
    deferred: usize,
    dropped: usize,
    slow_callbacks: usize,
    queue_depth: usize,
    pending_sequence_span: u64,
    runtime_remaining_deliveries: usize,
    runtime_oldest_pending_age_millis: u64,
}

impl EditorRuntimeEventPumpReport {
    pub const fn applied(self) -> usize {
        self.applied
    }

    pub const fn drained(self) -> usize {
        self.drained
    }

    pub const fn drained_encoded_bytes(self) -> usize {
        self.drained_encoded_bytes
    }

    pub const fn runtime_drain_elapsed(self) -> Duration {
        self.runtime_drain_elapsed
    }

    pub const fn runtime_drain_p95(self) -> Duration {
        self.runtime_drain_p95
    }

    pub const fn decode_elapsed(self) -> Duration {
        self.decode_elapsed
    }

    pub const fn decode_p95(self) -> Duration {
        self.decode_p95
    }

    pub const fn deferred(self) -> usize {
        self.deferred
    }

    pub const fn dropped(self) -> usize {
        self.dropped
    }

    pub const fn slow_callbacks(self) -> usize {
        self.slow_callbacks
    }

    pub const fn queue_depth(self) -> usize {
        self.queue_depth
    }

    pub const fn pending_sequence_span(self) -> u64 {
        self.pending_sequence_span
    }

    pub const fn runtime_remaining_deliveries(self) -> usize {
        self.runtime_remaining_deliveries
    }

    pub const fn runtime_oldest_pending_age_millis(self) -> u64 {
        self.runtime_oldest_pending_age_millis
    }

    pub(super) fn record_drained_page(
        &mut self,
        count: usize,
        encoded_bytes: usize,
        runtime_drain_elapsed: Duration,
        decode_elapsed: Duration,
        runtime_remaining_deliveries: usize,
        runtime_oldest_pending_age_millis: u64,
    ) {
        self.drained = self.drained.saturating_add(count);
        self.drained_encoded_bytes = self.drained_encoded_bytes.saturating_add(encoded_bytes);
        self.runtime_drain_elapsed = self
            .runtime_drain_elapsed
            .saturating_add(runtime_drain_elapsed);
        self.decode_elapsed = self.decode_elapsed.saturating_add(decode_elapsed);
        self.runtime_remaining_deliveries = self
            .runtime_remaining_deliveries
            .saturating_add(runtime_remaining_deliveries);
        self.runtime_oldest_pending_age_millis = self
            .runtime_oldest_pending_age_millis
            .max(runtime_oldest_pending_age_millis);
    }

    pub(super) fn record_applied(
        &mut self,
        callback_elapsed: Duration,
        slow_callback_threshold: Duration,
    ) {
        self.applied = self.applied.saturating_add(1);
        if callback_elapsed > slow_callback_threshold {
            self.slow_callbacks = self.slow_callbacks.saturating_add(1);
        }
    }

    pub(super) fn record_dropped(&mut self, count: usize) {
        self.dropped = self.dropped.saturating_add(count);
    }

    pub(super) fn set_queue_pressure(&mut self, queue_depth: usize, pending_sequence_span: u64) {
        self.queue_depth = queue_depth;
        self.deferred = queue_depth;
        self.pending_sequence_span = pending_sequence_span;
    }

    pub(super) fn set_drain_percentiles(
        &mut self,
        runtime_drain_p95: Duration,
        decode_p95: Duration,
    ) {
        self.runtime_drain_p95 = runtime_drain_p95;
        self.decode_p95 = decode_p95;
    }
}
