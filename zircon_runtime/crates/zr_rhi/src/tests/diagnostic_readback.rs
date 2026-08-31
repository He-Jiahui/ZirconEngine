use crate::{
    DeviceGeneration, DeviceId, DiagnosticReadbackAdmission, DiagnosticReadbackBudget,
    DiagnosticReadbackError, DiagnosticReadbackKind, DiagnosticReadbackTerminal,
    DiagnosticReadbackTracker, RenderQueueClass, SubmissionTicket,
};

fn tracker(budget: DiagnosticReadbackBudget) -> DiagnosticReadbackTracker {
    DiagnosticReadbackTracker::new(DeviceId::new(9), DeviceGeneration::initial(), budget)
}

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(9),
        DeviceGeneration::initial(),
        RenderQueueClass::Copy,
        sequence,
    )
}

#[test]
fn diagnostic_readback_quota_rejection_emits_an_over_budget_receipt_without_gpu_work() {
    let mut tracker = tracker(DiagnosticReadbackBudget::new(1, 1, 4, 4, 4, 2));
    tracker.begin_frame(13).unwrap();

    let admission = tracker
        .admit_or_reject(DiagnosticReadbackKind::Buffer, 8)
        .unwrap();
    let DiagnosticReadbackAdmission::Rejected(receipt) = admission else {
        panic!("the oversized request must be terminalized at admission");
    };

    assert_eq!(receipt.terminal(), DiagnosticReadbackTerminal::OverBudget);
    assert_eq!(receipt.frame_key(), None);
    assert_eq!(tracker.pending_request_count(), 0);
    assert_eq!(tracker.take_completed_receipt(), Some(receipt));
}

#[test]
fn diagnostic_readback_admission_enforces_request_frame_and_resident_quotas() {
    let budget = DiagnosticReadbackBudget::new(2, 3, 8, 12, 16, 2);
    let mut tracker = tracker(budget);
    tracker.begin_frame(12).unwrap();

    assert!(matches!(
        tracker.admit(DiagnosticReadbackKind::Buffer, 0),
        Err(DiagnosticReadbackError::EmptyRequest)
    ));
    assert!(matches!(
        tracker.admit(DiagnosticReadbackKind::Buffer, 9),
        Err(DiagnosticReadbackError::RequestBytesExceeded { .. })
    ));

    let first = tracker.admit(DiagnosticReadbackKind::Buffer, 8).unwrap();
    assert!(matches!(
        tracker.admit(DiagnosticReadbackKind::Timestamp, 8),
        Err(DiagnosticReadbackError::FrameBytesExceeded { .. })
    ));
    let second = tracker.admit(DiagnosticReadbackKind::Timestamp, 4).unwrap();
    assert!(matches!(
        tracker.admit(DiagnosticReadbackKind::PipelineStatistics, 4),
        Err(DiagnosticReadbackError::FrameRequestLimitExceeded { .. })
    ));

    assert_eq!(
        tracker.bind_active_frame(ticket(1)).unwrap().submission(),
        ticket(1)
    );
    tracker.begin_frame(13).unwrap();
    let third = tracker.admit(DiagnosticReadbackKind::Texture, 4).unwrap();
    assert!(matches!(
        tracker.admit(DiagnosticReadbackKind::Texture, 4),
        Err(DiagnosticReadbackError::PendingRequestLimitExceeded { .. })
    ));
    assert_eq!(tracker.pending_request_count(), 3);
    assert_eq!(tracker.pending_bytes(), 16);

    assert!(tracker
        .terminalize(first, DiagnosticReadbackTerminal::Succeeded)
        .is_some());
    assert!(tracker
        .terminalize(second, DiagnosticReadbackTerminal::Succeeded)
        .is_some());
    assert!(tracker
        .terminalize(third, DiagnosticReadbackTerminal::Succeeded)
        .is_some());
}

#[test]
fn diagnostic_readback_frame_key_rejects_cross_device_or_generation_submission_tickets() {
    let mut tracker = tracker(DiagnosticReadbackBudget::default());
    tracker.begin_frame(41).unwrap();
    let request = tracker.admit(DiagnosticReadbackKind::Timestamp, 8).unwrap();
    let foreign_ticket = SubmissionTicket::new(
        DeviceId::new(10),
        DeviceGeneration::initial(),
        RenderQueueClass::Copy,
        1,
    );

    assert!(matches!(
        tracker.bind_active_frame(foreign_ticket),
        Err(DiagnosticReadbackError::SubmissionIdentityMismatch { .. })
    ));
    let key = tracker.bind_active_frame(ticket(2)).unwrap();
    assert_eq!(key.device_id(), DeviceId::new(9));
    assert_eq!(key.generation(), DeviceGeneration::initial());
    assert_eq!(key.submission(), ticket(2));

    let receipt = tracker
        .terminalize(request, DiagnosticReadbackTerminal::Cancelled)
        .expect("the pending request must receive its first terminal receipt");
    assert_eq!(receipt.frame_key(), Some(key));
    assert_eq!(receipt.terminal(), DiagnosticReadbackTerminal::Cancelled);
    assert!(tracker
        .terminalize(request, DiagnosticReadbackTerminal::Cancelled)
        .is_none());
}

#[test]
fn diagnostic_readback_failure_paths_emit_exactly_once_and_keep_a_bounded_result_ring() {
    let mut tracker = tracker(DiagnosticReadbackBudget::new(8, 8, 64, 64, 64, 1));
    tracker.begin_frame(1).unwrap();
    let first = tracker.admit(DiagnosticReadbackKind::Buffer, 8).unwrap();
    let second = tracker
        .admit(DiagnosticReadbackKind::PipelineStatistics, 8)
        .unwrap();
    let frame_key = tracker.bind_active_frame(ticket(3)).unwrap();

    let lost = tracker.terminalize_frame(frame_key, DiagnosticReadbackTerminal::DeviceLost);
    assert_eq!(lost.len(), 2);
    assert!(lost
        .iter()
        .all(|receipt| receipt.terminal() == DiagnosticReadbackTerminal::DeviceLost));
    assert!(tracker
        .terminalize(first, DiagnosticReadbackTerminal::DeviceLost)
        .is_none());
    assert!(tracker
        .terminalize(second, DiagnosticReadbackTerminal::Shutdown)
        .is_none());
    assert_eq!(tracker.pending_request_count(), 0);
    assert_eq!(tracker.completed_receipt_count(), 1);
    assert_eq!(tracker.dropped_completed_receipt_count(), 1);

    let drained = tracker.take_completed_receipt().unwrap();
    assert_eq!(drained.terminal(), DiagnosticReadbackTerminal::DeviceLost);
    assert!(tracker.take_completed_receipt().is_none());
}

#[test]
fn diagnostic_readback_unbound_active_frame_can_terminalize_after_native_encode_failure() {
    let mut tracker = tracker(DiagnosticReadbackBudget::default());
    tracker.begin_frame(31).unwrap();
    let request = tracker.admit(DiagnosticReadbackKind::Buffer, 16).unwrap();

    let receipts = tracker.terminalize_active_frame(DiagnosticReadbackTerminal::MapFailed);

    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request(), request);
    assert_eq!(receipts[0].frame_key(), None);
    assert_eq!(
        receipts[0].terminal(),
        DiagnosticReadbackTerminal::MapFailed
    );
    assert_eq!(tracker.pending_request_count(), 0);
    assert!(tracker.begin_frame(32).is_ok());
}

#[test]
fn diagnostic_readback_shutdown_terminalizes_active_and_submitted_requests_without_duplicates() {
    let mut tracker = tracker(DiagnosticReadbackBudget::default());
    tracker.begin_frame(7).unwrap();
    let active = tracker.admit(DiagnosticReadbackKind::Texture, 16).unwrap();
    let active_receipts = tracker.terminalize_all(DiagnosticReadbackTerminal::Cancelled);
    assert_eq!(active_receipts.len(), 1);
    assert_eq!(active_receipts[0].request(), active);

    tracker.begin_frame(8).unwrap();
    let submitted = tracker.admit(DiagnosticReadbackKind::Buffer, 16).unwrap();
    tracker.bind_active_frame(ticket(4)).unwrap();
    let receipts = tracker.terminalize_all(DiagnosticReadbackTerminal::Shutdown);

    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request(), submitted);
    assert_eq!(receipts[0].terminal(), DiagnosticReadbackTerminal::Shutdown);
    assert!(tracker
        .terminalize(submitted, DiagnosticReadbackTerminal::Shutdown)
        .is_none());
}
