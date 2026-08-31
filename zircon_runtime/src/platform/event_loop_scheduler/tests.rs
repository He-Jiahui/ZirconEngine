use std::time::{Duration, Instant};

use crate::core::framework::platform::{
    EventLoopBackgroundPolicy, EventLoopClockDomain, EventLoopControlFlow, EventLoopHostWakeReason,
    EventLoopWakeRequest, EventLoopWakeSource,
};

use super::EventLoopScheduler;

#[test]
fn scheduler_replaces_each_source_deadline_and_tracks_strict_overdue_dispatches() {
    let now = Instant::now();
    let mut scheduler = EventLoopScheduler::default();
    scheduler.schedule(EventLoopWakeRequest::at(
        EventLoopWakeSource::Timer,
        EventLoopClockDomain::Monotonic,
        now + Duration::from_millis(20),
    ));
    scheduler.schedule(EventLoopWakeRequest::at(
        EventLoopWakeSource::FrameDemand,
        EventLoopClockDomain::Monotonic,
        now + Duration::from_millis(10),
    ));
    scheduler.schedule(EventLoopWakeRequest::at(
        EventLoopWakeSource::FrameDemand,
        EventLoopClockDomain::Monotonic,
        now + Duration::from_millis(30),
    ));

    assert_eq!(
        scheduler.control_flow(now),
        EventLoopControlFlow::WaitUntil {
            domain: EventLoopClockDomain::Monotonic,
            deadline: now + Duration::from_millis(20),
        }
    );
    let timer = scheduler.take_due(now + Duration::from_millis(20));
    assert!(timer.contains(EventLoopWakeSource::Timer));
    assert!(!timer.contains(EventLoopWakeSource::FrameDemand));

    let frame = scheduler.take_due(now + Duration::from_millis(31));
    assert!(frame.contains(EventLoopWakeSource::FrameDemand));
    let snapshot = scheduler.snapshot();
    assert_eq!(snapshot.replaced_requests(), 1);
    assert_eq!(snapshot.overdue_dispatches(), 1);
    assert_eq!(snapshot.maximum_lateness(), Duration::from_millis(1));
}

#[test]
fn scheduler_reports_backlog_and_immediate_wakes_without_allocating_a_queue() {
    let now = Instant::now();
    let mut scheduler = EventLoopScheduler::default();
    scheduler.observe_backlog(EventLoopWakeSource::HostCommand, 3);
    scheduler.observe_backlog(EventLoopWakeSource::Input, 2);
    scheduler.schedule(EventLoopWakeRequest::immediate(
        EventLoopWakeSource::HostCommand,
        now,
    ));

    assert_eq!(scheduler.control_flow(now), EventLoopControlFlow::Poll);
    let due = scheduler.take_due(now);
    assert!(due.contains(EventLoopWakeSource::HostCommand));
    let snapshot = scheduler.snapshot();
    assert_eq!(snapshot.backlog(), 5);
    assert_eq!(snapshot.backlog_high_watermark(), 5);
    assert_eq!(snapshot.pending_sources(), 0);
}

#[test]
fn scheduler_records_source_level_deadline_lateness_and_starvation_prevention() {
    let now = Instant::now();
    let mut scheduler = EventLoopScheduler::default();
    scheduler.schedule(EventLoopWakeRequest::immediate(
        EventLoopWakeSource::FrameDemand,
        now,
    ));
    scheduler.schedule(EventLoopWakeRequest::immediate(
        EventLoopWakeSource::HostCommand,
        now,
    ));

    let due = scheduler.take_due(now + Duration::from_millis(3));
    assert!(due.contains(EventLoopWakeSource::FrameDemand));
    assert!(due.contains(EventLoopWakeSource::HostCommand));
    assert_eq!(due.count(), 2);

    let snapshot = scheduler.snapshot();
    assert_eq!(snapshot.starvation_preventions(), 1);
    for source in [
        EventLoopWakeSource::FrameDemand,
        EventLoopWakeSource::HostCommand,
    ] {
        let evidence = snapshot
            .last_dispatch(source)
            .expect("each due source retains dispatch evidence");
        assert_eq!(evidence.source(), source);
        assert_eq!(evidence.deadline(), now);
        assert_eq!(evidence.dispatched_at(), now + Duration::from_millis(3));
        assert_eq!(evidence.lateness(), Duration::from_millis(3));
    }
}

#[test]
fn scheduler_retains_background_policy_and_per_source_backlog_for_host_audit() {
    let now = Instant::now();
    let mut scheduler = EventLoopScheduler::default();
    scheduler.observe_background_policy(EventLoopBackgroundPolicy::Throttled);
    scheduler.observe_backlog(EventLoopWakeSource::Background, 4);
    scheduler.schedule(EventLoopWakeRequest::at(
        EventLoopWakeSource::Background,
        EventLoopClockDomain::Monotonic,
        now + Duration::from_millis(50),
    ));

    let snapshot = scheduler.snapshot();
    assert_eq!(
        snapshot.background_policy(),
        Some(EventLoopBackgroundPolicy::Throttled)
    );
    assert_eq!(snapshot.background_policy_transitions(), 1);
    assert_eq!(snapshot.backlog_for(EventLoopWakeSource::Background), 4);
    assert_eq!(
        snapshot.pending_deadline(EventLoopWakeSource::Background),
        Some(now + Duration::from_millis(50))
    );

    scheduler.observe_background_policy(EventLoopBackgroundPolicy::Suspended);
    assert_eq!(
        scheduler.snapshot().background_policy(),
        Some(EventLoopBackgroundPolicy::Suspended)
    );
    assert_eq!(scheduler.snapshot().background_policy_transitions(), 2);
}

#[test]
fn scheduler_records_host_wake_reasons_without_conflating_them_with_work_sources() {
    let now = Instant::now();
    let mut scheduler = EventLoopScheduler::default();
    scheduler.observe_host_wake(EventLoopHostWakeReason::Initialization, now);
    scheduler.observe_host_wake(
        EventLoopHostWakeReason::WaitCancelled,
        now + Duration::from_millis(2),
    );
    scheduler.observe_host_wake(
        EventLoopHostWakeReason::WaitCancelled,
        now + Duration::from_millis(4),
    );

    let snapshot = scheduler.snapshot();
    assert_eq!(
        snapshot.host_wake_count(EventLoopHostWakeReason::Initialization),
        1
    );
    assert_eq!(
        snapshot.host_wake_count(EventLoopHostWakeReason::WaitCancelled),
        2
    );
    let last = snapshot
        .last_host_wake()
        .expect("latest host wake observation is retained");
    assert_eq!(last.reason(), EventLoopHostWakeReason::WaitCancelled);
    assert_eq!(last.observed_at(), now + Duration::from_millis(4));
}
