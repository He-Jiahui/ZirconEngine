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

/// Runtime backlog evidence accumulated from the consumers sampled during one pump.
///
/// The remaining-delivery total is a lower bound whenever one or more active consumers have
/// not been sampled. This keeps known runtime pressure observable without overstating it as a
/// complete snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorRuntimeEventBacklogObservation {
    known_remaining_deliveries_lower_bound: usize,
    sampled_consumer_count: usize,
    unknown_consumer_count: usize,
    max_oldest_pending_age_millis: Option<u64>,
    max_observation_age: Option<Duration>,
}

impl EditorRuntimeEventBacklogObservation {
    pub const fn known_remaining_deliveries_lower_bound(self) -> usize {
        self.known_remaining_deliveries_lower_bound
    }

    pub const fn sampled_consumer_count(self) -> usize {
        self.sampled_consumer_count
    }

    pub const fn unknown_consumer_count(self) -> usize {
        self.unknown_consumer_count
    }

    pub const fn max_oldest_pending_age_millis(self) -> Option<u64> {
        self.max_oldest_pending_age_millis
    }

    pub const fn max_observation_age(self) -> Option<Duration> {
        self.max_observation_age
    }

    pub const fn is_complete(self) -> bool {
        self.unknown_consumer_count == 0
    }

    pub(super) fn record_sample(
        &mut self,
        remaining_deliveries: usize,
        oldest_pending_age_millis: u64,
        observation_age: Duration,
    ) {
        self.known_remaining_deliveries_lower_bound = self
            .known_remaining_deliveries_lower_bound
            .saturating_add(remaining_deliveries);
        self.sampled_consumer_count = self.sampled_consumer_count.saturating_add(1);
        self.max_oldest_pending_age_millis = Some(
            self.max_oldest_pending_age_millis
                .unwrap_or_default()
                .max(oldest_pending_age_millis),
        );
        self.max_observation_age = Some(
            self.max_observation_age
                .unwrap_or_default()
                .max(observation_age),
        );
    }

    pub(super) fn record_unknown_consumer(&mut self) {
        self.unknown_consumer_count = self.unknown_consumer_count.saturating_add(1);
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
    stale_consumers: usize,
    queue_depth: usize,
    pending_sequence_span: u64,
    pending_encoded_bytes_upper_bound: usize,
    pending_oldest_age: Duration,
    runtime_backlog_observation: EditorRuntimeEventBacklogObservation,
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

    /// Number of active consumers retired because their origin transport was replaced.
    pub const fn stale_consumers(self) -> usize {
        self.stale_consumers
    }

    pub const fn queue_depth(self) -> usize {
        self.queue_depth
    }

    pub const fn pending_sequence_span(self) -> u64 {
        self.pending_sequence_span
    }

    /// Conservative encoded-byte bound for deliveries retained by the editor host.
    ///
    /// The host retains at most one decoded runtime page for each consumer, so a partly
    /// consumed page still reports its original encoded size until it is fully released.
    pub const fn pending_encoded_bytes_upper_bound(self) -> usize {
        self.pending_encoded_bytes_upper_bound
    }

    pub const fn pending_oldest_age(self) -> Duration {
        self.pending_oldest_age
    }

    pub const fn runtime_backlog_observation(self) -> EditorRuntimeEventBacklogObservation {
        self.runtime_backlog_observation
    }

    pub(super) fn record_drained_page(
        &mut self,
        count: usize,
        encoded_bytes: usize,
        runtime_drain_elapsed: Duration,
        decode_elapsed: Duration,
    ) {
        self.drained = self.drained.saturating_add(count);
        self.drained_encoded_bytes = self.drained_encoded_bytes.saturating_add(encoded_bytes);
        self.runtime_drain_elapsed = self
            .runtime_drain_elapsed
            .saturating_add(runtime_drain_elapsed);
        self.decode_elapsed = self.decode_elapsed.saturating_add(decode_elapsed);
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

    pub(super) fn record_stale_consumer(&mut self) {
        self.stale_consumers = self.stale_consumers.saturating_add(1);
    }

    pub(super) fn set_queue_pressure(
        &mut self,
        queue_depth: usize,
        pending_sequence_span: u64,
        pending_encoded_bytes_upper_bound: usize,
        pending_oldest_age: Duration,
    ) {
        self.queue_depth = queue_depth;
        self.deferred = queue_depth;
        self.pending_sequence_span = pending_sequence_span;
        self.pending_encoded_bytes_upper_bound = pending_encoded_bytes_upper_bound;
        self.pending_oldest_age = pending_oldest_age;
    }

    pub(super) fn set_runtime_backlog_observation(
        &mut self,
        runtime_backlog_observation: EditorRuntimeEventBacklogObservation,
    ) {
        self.runtime_backlog_observation = runtime_backlog_observation;
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
