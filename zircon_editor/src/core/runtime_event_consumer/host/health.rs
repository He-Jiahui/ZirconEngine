use std::time::Duration;

const DEFAULT_MAX_CONSECUTIVE_FAILURES: u32 = 3;
const DEFAULT_FAILURE_RATE_WINDOW_ATTEMPTS: u32 = 8;
const DEFAULT_MAX_FAILURES_PER_WINDOW: u32 = 4;
const DEFAULT_MAX_CONSECUTIVE_SLOW_CALLBACKS: u32 = 3;

/// Bounded host policy for isolating a consumer that repeatedly harms frame progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorRuntimeEventConsumerFaultPolicy {
    max_consecutive_failures: u32,
    failure_rate_window_attempts: u32,
    max_failures_per_window: u32,
    max_consecutive_slow_callbacks: u32,
}

impl EditorRuntimeEventConsumerFaultPolicy {
    pub const fn new(
        max_consecutive_failures: u32,
        failure_rate_window_attempts: u32,
        max_failures_per_window: u32,
        max_consecutive_slow_callbacks: u32,
    ) -> Self {
        let failure_rate_window_attempts = failure_rate_window_attempts.max(1);
        Self {
            max_consecutive_failures: max_consecutive_failures.max(1),
            failure_rate_window_attempts,
            max_failures_per_window: max_failures_per_window
                .max(1)
                .min(failure_rate_window_attempts),
            max_consecutive_slow_callbacks: max_consecutive_slow_callbacks.max(1),
        }
    }

    pub const fn max_consecutive_failures(self) -> u32 {
        self.max_consecutive_failures
    }

    pub const fn failure_rate_window_attempts(self) -> u32 {
        self.failure_rate_window_attempts
    }

    pub const fn max_failures_per_window(self) -> u32 {
        self.max_failures_per_window
    }

    pub const fn max_consecutive_slow_callbacks(self) -> u32 {
        self.max_consecutive_slow_callbacks
    }
}

impl Default for EditorRuntimeEventConsumerFaultPolicy {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_CONSECUTIVE_FAILURES,
            DEFAULT_FAILURE_RATE_WINDOW_ATTEMPTS,
            DEFAULT_MAX_FAILURES_PER_WINDOW,
            DEFAULT_MAX_CONSECUTIVE_SLOW_CALLBACKS,
        )
    }
}

/// Explains the terminal policy decision exposed to plugin diagnostics and retry controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorRuntimeEventConsumerQuarantineReason {
    CallbackPanicked,
    ConsecutiveFailures,
    FailureRateExceeded,
    ConsecutiveSlowCallbacks,
}

#[derive(Default)]
pub(super) struct ConsumerCallbackHealth {
    consecutive_failures: u32,
    failure_window_attempts: u32,
    failure_window_failures: u32,
    consecutive_slow_callbacks: u32,
}

impl ConsumerCallbackHealth {
    pub(super) fn record(
        &mut self,
        policy: EditorRuntimeEventConsumerFaultPolicy,
        failed: bool,
        callback_elapsed: Duration,
        slow_callback_threshold: Duration,
    ) -> Option<EditorRuntimeEventConsumerQuarantineReason> {
        self.consecutive_failures = if failed {
            self.consecutive_failures.saturating_add(1)
        } else {
            0
        };
        self.consecutive_slow_callbacks = if callback_elapsed > slow_callback_threshold {
            self.consecutive_slow_callbacks.saturating_add(1)
        } else {
            0
        };
        self.failure_window_attempts = self.failure_window_attempts.saturating_add(1);
        if failed {
            self.failure_window_failures = self.failure_window_failures.saturating_add(1);
        }

        let failure_rate_exceeded = self.failure_window_attempts
            >= policy.failure_rate_window_attempts()
            && self.failure_window_failures >= policy.max_failures_per_window();
        if self.failure_window_attempts >= policy.failure_rate_window_attempts() {
            self.failure_window_attempts = 0;
            self.failure_window_failures = 0;
        }

        if self.consecutive_failures >= policy.max_consecutive_failures() {
            Some(EditorRuntimeEventConsumerQuarantineReason::ConsecutiveFailures)
        } else if failure_rate_exceeded {
            Some(EditorRuntimeEventConsumerQuarantineReason::FailureRateExceeded)
        } else if self.consecutive_slow_callbacks >= policy.max_consecutive_slow_callbacks() {
            Some(EditorRuntimeEventConsumerQuarantineReason::ConsecutiveSlowCallbacks)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        ConsumerCallbackHealth, EditorRuntimeEventConsumerFaultPolicy,
        EditorRuntimeEventConsumerQuarantineReason,
    };

    #[test]
    fn normalizes_zero_fault_limits_and_quarantines_on_the_first_failure() {
        let policy = EditorRuntimeEventConsumerFaultPolicy::new(0, 0, 0, 0);
        assert_eq!(policy.max_consecutive_failures(), 1);
        assert_eq!(policy.failure_rate_window_attempts(), 1);
        assert_eq!(policy.max_failures_per_window(), 1);
        assert_eq!(policy.max_consecutive_slow_callbacks(), 1);
        assert_eq!(
            ConsumerCallbackHealth::default().record(
                policy,
                true,
                Duration::ZERO,
                Duration::from_millis(1),
            ),
            Some(EditorRuntimeEventConsumerQuarantineReason::ConsecutiveFailures)
        );
    }

    #[test]
    fn failure_rate_window_detects_intermittent_failures_without_a_history_scan() {
        let policy = EditorRuntimeEventConsumerFaultPolicy::new(4, 3, 2, 4);
        let mut health = ConsumerCallbackHealth::default();
        assert_eq!(
            health.record(policy, true, Duration::ZERO, Duration::from_millis(1)),
            None
        );
        assert_eq!(
            health.record(policy, false, Duration::ZERO, Duration::from_millis(1)),
            None
        );
        assert_eq!(
            health.record(policy, true, Duration::ZERO, Duration::from_millis(1)),
            Some(EditorRuntimeEventConsumerQuarantineReason::FailureRateExceeded)
        );
    }
}
