use super::{
    asset_refresh_commit_reason, AssetRefreshAccumulator, AssetRefreshCommitReason,
    AssetRefreshEvents, ASSET_REFRESH_QUIET_PERIOD, MAX_ACCUMULATED_ASSET_REFRESH_EVENTS,
    MAX_ASSET_REFRESH_DEFERRAL,
};
use std::time::{Duration, Instant};

#[test]
fn partial_backlog_waits_for_more_events_inside_the_bounded_window() {
    assert_eq!(
        asset_refresh_commit_reason(12, true, Duration::from_millis(4), Duration::from_millis(4),),
        None
    );
}

#[test]
fn drained_queue_waits_for_a_quiet_period_before_committing() {
    assert_eq!(
        asset_refresh_commit_reason(
            12,
            false,
            Duration::from_millis(4),
            ASSET_REFRESH_QUIET_PERIOD - Duration::from_millis(1),
        ),
        None
    );
    assert_eq!(
        asset_refresh_commit_reason(
            12,
            false,
            ASSET_REFRESH_QUIET_PERIOD,
            ASSET_REFRESH_QUIET_PERIOD,
        ),
        Some(AssetRefreshCommitReason::QueueQuiesced)
    );
}

#[test]
fn capacity_and_latency_keep_continuous_streams_bounded() {
    assert_eq!(
        asset_refresh_commit_reason(
            MAX_ACCUMULATED_ASSET_REFRESH_EVENTS,
            true,
            Duration::ZERO,
            Duration::ZERO,
        ),
        Some(AssetRefreshCommitReason::Capacity)
    );
    assert_eq!(
        asset_refresh_commit_reason(1, false, MAX_ASSET_REFRESH_DEFERRAL, Duration::ZERO,),
        Some(AssetRefreshCommitReason::MaxDeferral)
    );
}

#[test]
fn max_deferral_does_not_commit_while_a_bulk_backlog_is_still_draining() {
    assert_eq!(
        asset_refresh_commit_reason(128, true, MAX_ASSET_REFRESH_DEFERRAL, Duration::ZERO,),
        None
    );
}

#[test]
fn resource_lag_is_coalesced_but_preserved_for_the_eventual_reconciliation() {
    let now = Instant::now();
    let mut accumulator = AssetRefreshAccumulator::default();
    let lagged = AssetRefreshEvents {
        resource_generation_lagged: true,
        ..Default::default()
    };

    assert!(accumulator.accumulate(lagged, true, now).is_none());
    let committed = accumulator
        .accumulate(
            AssetRefreshEvents::default(),
            false,
            now + MAX_ASSET_REFRESH_DEFERRAL,
        )
        .expect("max deferral must eventually commit a lag reconciliation");
    assert!(committed.resource_generation_lagged);
}

#[test]
fn pending_events_expose_the_earliest_quiet_or_max_deferral_deadline() {
    let now = Instant::now();
    let mut accumulator = AssetRefreshAccumulator::default();
    let lagged = AssetRefreshEvents {
        resource_generation_lagged: true,
        ..Default::default()
    };

    assert!(accumulator.accumulate(lagged, false, now).is_none());
    assert_eq!(
        accumulator.next_commit_deadline(),
        Some(now + ASSET_REFRESH_QUIET_PERIOD)
    );

    let later = now + MAX_ASSET_REFRESH_DEFERRAL - Duration::from_millis(1);
    assert!(accumulator
        .accumulate(
            AssetRefreshEvents {
                resource_generation_lagged: true,
                ..Default::default()
            },
            false,
            later,
        )
        .is_none());
    assert_eq!(
        accumulator.next_commit_deadline(),
        Some(now + MAX_ASSET_REFRESH_DEFERRAL)
    );
}

#[test]
fn superseded_active_scene_reload_is_coalesced_into_one_bounded_commit() {
    let now = Instant::now();
    let mut accumulator = AssetRefreshAccumulator::default();

    assert_eq!(
        accumulator.request_active_scene_reload(now),
        now + ASSET_REFRESH_QUIET_PERIOD
    );
    assert!(accumulator
        .accumulate(
            AssetRefreshEvents::default(),
            false,
            now + ASSET_REFRESH_QUIET_PERIOD - Duration::from_millis(1),
        )
        .is_none());
    let committed = accumulator
        .accumulate(
            AssetRefreshEvents::default(),
            false,
            now + ASSET_REFRESH_QUIET_PERIOD,
        )
        .expect("the synthetic reload must commit after the quiet period");

    assert!(committed.active_scene_reload_requested);
    assert!(accumulator.next_commit_deadline().is_none());
}

#[test]
fn resource_sequence_exhaustion_is_latched_once_for_the_host_lifetime() {
    let mut accumulator = AssetRefreshAccumulator::default();

    assert!(accumulator.latch_resource_sequence_exhaustion());
    assert!(!accumulator.latch_resource_sequence_exhaustion());
    assert!(accumulator.resource_sequence_exhausted());
}
