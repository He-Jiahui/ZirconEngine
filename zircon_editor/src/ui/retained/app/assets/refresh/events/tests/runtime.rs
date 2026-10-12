use std::time::{Duration, Instant};

use super::{
    can_drain_more, resource_event_drain_action, ResourceEventDrainAction,
    MAX_ASSET_REFRESH_EVENTS_PER_STREAM,
};
use zircon_runtime::core::resource::{ResourceEventGap, ResourceEventTryRecvError};

#[test]
fn per_stream_count_budget_is_a_hard_upper_bound() {
    let now = Instant::now();
    assert!(can_drain_more(now, MAX_ASSET_REFRESH_EVENTS_PER_STREAM - 1));
    assert!(!can_drain_more(now, MAX_ASSET_REFRESH_EVENTS_PER_STREAM));
}

#[test]
fn elapsed_stream_budget_stops_drain_before_count_limit() {
    let now = Instant::now();
    let expired = now - Duration::from_millis(3);
    assert!(!can_drain_more(expired, 0));
}

#[test]
fn every_stream_uses_an_independent_time_slice() {
    let expired_stream = Instant::now() - Duration::from_millis(3);
    let next_stream = Instant::now();

    assert!(!can_drain_more(expired_stream, 0));
    assert!(can_drain_more(next_stream, 0));
}

#[test]
fn resource_sequence_exhaustion_requests_one_terminal_reconciliation() {
    assert_eq!(
        resource_event_drain_action(ResourceEventTryRecvError::SequenceExhausted),
        ResourceEventDrainAction::ReconcileAndStop
    );
}

#[test]
fn recoverable_lag_continues_but_empty_and_disconnect_stop_without_reconciliation() {
    assert_eq!(
        resource_event_drain_action(ResourceEventTryRecvError::Lagged(ResourceEventGap {
            expected_sequence: 7,
            oldest_available_sequence: Some(11),
        })),
        ResourceEventDrainAction::ReconcileAndContinue
    );
    assert_eq!(
        resource_event_drain_action(ResourceEventTryRecvError::Empty),
        ResourceEventDrainAction::Stop
    );
    assert_eq!(
        resource_event_drain_action(ResourceEventTryRecvError::Disconnected),
        ResourceEventDrainAction::Stop
    );
}
