use super::*;

#[test]
fn preference_persistence_lane_cancelled_pre_fence_obligation_fails_fence() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 1024), scheduler);
    let admission = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let authority = admission.cancel_authority();
    let ticket = admission.activate();
    ticket.cancel_before_start(&authority).unwrap();
    let fence = lane
        .submit_fence(1, BoundedKeyedIoWorkDeadline::none(), Box::new(|| Ok(())))
        .unwrap();
    assert_eq!(
        ticket.terminal(),
        Some(BoundedKeyedIoTerminal::CancelledBeforeStart)
    );
    release_tx.send(()).unwrap();
    assert!(matches!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(BoundedKeyedIoFailure {
            code: "pre_fence_obligation_cancelled_before_start"
        }))
    ));
    blocker.wait();
}

#[test]
fn preference_persistence_lane_preserves_pre_fence_failure_code() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 1024), scheduler);
    let work = lane
        .try_admit(
            "key",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Err(BoundedKeyedIoFailure::new("preference_backend_denied"))),
        )
        .unwrap()
        .activate();
    let fence = lane
        .submit_fence(1, BoundedKeyedIoWorkDeadline::none(), Box::new(|| Ok(())))
        .unwrap();

    release_tx.send(()).unwrap();
    assert_eq!(
        work.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(
            BoundedKeyedIoFailure::new("preference_backend_denied")
        ))
    );
    assert_eq!(
        fence
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(
            BoundedKeyedIoFailure::new("preference_backend_denied")
        ))
    );
    blocker.wait();
}
