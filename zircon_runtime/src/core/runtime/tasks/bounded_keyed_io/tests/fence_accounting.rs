use super::*;

#[test]
fn preference_persistence_lane_charges_fence_prerequisites_before_capture() {
    let lane = lane(BoundedKeyedIoLimits::new(4, 2));
    let admission = lane
        .try_admit(
            "pre-fence",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let authority = admission.cancel_authority();
    let ticket = admission.ticket();

    assert!(matches!(
        lane.submit_fence(1, BoundedKeyedIoWorkDeadline::none(), Box::new(|| Ok(())),),
        Err(BoundedKeyedIoAdmissionError::RetainedBytesCapacityExceeded)
    ));
    assert_eq!(lane.diagnostics().queue_entries, 1);
    assert_eq!(lane.diagnostics().retained_bytes, 1);
    assert_eq!(ticket.cancel_before_start(&authority), Ok(()));
    drop(admission);

    let retry = lane
        .try_admit(
            "post-fence-rejection",
            2,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    assert_eq!(retry.epoch(), GlobalAdmissionEpoch::initial());
}

#[test]
fn preference_persistence_lane_consecutive_fences_retain_linear_prerequisite_records() {
    const FENCE_COUNT: usize = 256;
    const LINEAR_RETAINED_BYTE_LIMIT: usize = 64 * 1024;

    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(
        BoundedKeyedIoLimits::new(FENCE_COUNT, LINEAR_RETAINED_BYTE_LIMIT),
        scheduler,
    );
    let mut last_ticket = None;
    for _ in 0..FENCE_COUNT {
        last_ticket = Some(
            lane.submit_fence(1, BoundedKeyedIoWorkDeadline::none(), Box::new(|| Ok(())))
                .unwrap()
                .ticket(),
        );
    }

    let diagnostics = lane.diagnostics();
    assert_eq!(diagnostics.queue_entries, FENCE_COUNT);
    assert!(diagnostics.retained_bytes > FENCE_COUNT);
    assert!(diagnostics.retained_bytes <= LINEAR_RETAINED_BYTE_LIMIT);

    release_tx.send(()).unwrap();
    assert_eq!(
        last_ticket
            .unwrap()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    blocker.wait();
}
