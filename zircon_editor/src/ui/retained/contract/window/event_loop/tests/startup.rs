use super::*;

#[test]
fn a_never_ready_factory_has_a_bounded_poll_count_and_no_idle_wakes_after_timeout() {
    let mut startup = PresenterStartup::default();
    let start = Instant::now();
    let mut polls = 0;
    for elapsed in 0..=2_000 {
        match startup.action(start + Duration::from_millis(elapsed)) {
            PresenterStartupAction::Poll => polls += 1,
            PresenterStartupAction::TimedOut => assert_eq!(elapsed, 2_000),
            PresenterStartupAction::Wait => {}
            other => panic!("unexpected choice during readiness: {other:?}"),
        }
    }
    assert_eq!(polls, 40);
    assert_eq!(startup.wake_deadline(), None);
    for elapsed in [2_001, 5_000, 60_000] {
        assert_eq!(startup.action(start + Duration::from_millis(elapsed)),
            PresenterStartupAction::Selected { runtime: false });
        assert_eq!(startup.wake_deadline(), None);
    }
}

#[test]
fn a_ready_or_failed_factory_finishes_readiness_without_replacing_a_surface_later() {
    for runtime in [true, false] {
        let mut startup = PresenterStartup::default();
        let now = Instant::now();
        assert_eq!(startup.action(now), PresenterStartupAction::Poll);
        assert!(startup.wake_deadline().is_some());
        startup.select(runtime);
        startup.select(!runtime);
        assert_eq!(startup.action(now + Duration::from_secs(60)),
            PresenterStartupAction::Selected { runtime });
        assert_eq!(startup.wake_deadline(), None);
    }
}

#[test]
fn input_batches_do_not_accelerate_the_readiness_poll_rate() {
    let mut startup = PresenterStartup::default();
    let now = Instant::now();
    assert_eq!(startup.action(now), PresenterStartupAction::Poll);
    for _ in 0..1_000 {
        assert_eq!(startup.action(now), PresenterStartupAction::Wait);
    }
    assert_eq!(startup.action(now + READINESS_POLL_INTERVAL), PresenterStartupAction::Poll);
}
