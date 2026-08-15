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
    pending_encoded_bytes_upper_bound: usize,
    pending_oldest_age: Duration,
    last_observed_runtime_remaining_deliveries: Option<usize>,
    last_observed_runtime_oldest_pending_age_millis: Option<u64>,
    last_observed_runtime_backlog_observation_age: Option<Duration>,
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

    /// Total runtime backlog from the last complete per-consumer observation.
    ///
    /// A pump which only consumes editor-resident pending deliveries has not sampled runtime
    /// state again, so callers must not treat this value as a current runtime count.
    pub const fn last_observed_runtime_remaining_deliveries(self) -> Option<usize> {
        self.last_observed_runtime_remaining_deliveries
    }

    /// Oldest runtime backlog age reported with the last complete observation.
    pub const fn last_observed_runtime_oldest_pending_age_millis(self) -> Option<u64> {
        self.last_observed_runtime_oldest_pending_age_millis
    }

    /// Elapsed time since the oldest per-consumer runtime backlog observation in this report.
    pub const fn last_observed_runtime_backlog_observation_age(self) -> Option<Duration> {
        self.last_observed_runtime_backlog_observation_age
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

    pub(super) fn set_last_observed_runtime_backlog(
        &mut self,
        last_observed_runtime_backlog: Option<(usize, u64, Duration)>,
    ) {
        let Some((remaining_deliveries, oldest_pending_age_millis, observation_age)) =
            last_observed_runtime_backlog
        else {
            self.last_observed_runtime_remaining_deliveries = None;
            self.last_observed_runtime_oldest_pending_age_millis = None;
            self.last_observed_runtime_backlog_observation_age = None;
            return;
        };
        self.last_observed_runtime_remaining_deliveries = Some(remaining_deliveries);
        self.last_observed_runtime_oldest_pending_age_millis = Some(oldest_pending_age_millis);
        self.last_observed_runtime_backlog_observation_age = Some(observation_age);
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
