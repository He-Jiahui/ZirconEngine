use zr_rhi::{CommandList, RenderDevice, RenderQueueClass, RhiError, SubmissionStatus};

use super::{production_test_device, wait_for_submission};

#[test]
fn production_submission_packet_uses_one_ticket_for_multiple_command_lists() {
    let Some(device) = production_test_device() else {
        return;
    };
    let first = device
        .create_command_list(RenderQueueClass::Copy, "production-packet-first")
        .unwrap();
    let second = device
        .create_command_list(RenderQueueClass::Copy, "production-packet-second")
        .unwrap();
    let packet = device
        .create_submission_packet(RenderQueueClass::Copy, vec![first, second])
        .unwrap();

    assert_eq!(packet.command_list_count(), 2);
    let ticket = device.enqueue_submission_packet(packet).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, ticket);
}

#[test]
fn production_submission_packet_rejects_mixed_queue_classes_before_ticket_admission() {
    let Some(device) = production_test_device() else {
        return;
    };
    let copy = device
        .create_command_list(RenderQueueClass::Copy, "production-packet-copy")
        .unwrap();
    let compute = device
        .create_command_list(RenderQueueClass::Compute, "production-packet-compute")
        .unwrap();

    assert!(matches!(
        device.create_submission_packet(RenderQueueClass::Copy, vec![copy, compute]),
        Err(RhiError::SubmissionPacketQueueMismatch { .. })
    ));
}
