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
