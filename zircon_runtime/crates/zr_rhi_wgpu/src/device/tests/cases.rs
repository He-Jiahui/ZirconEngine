use super::*;

use std::sync::Arc;

#[test]
fn deterministic_rhi_contract_device_state_accessors_recover_poisoned_lock() {
    let device = DeterministicRhiContractDevice::new_headless();
    let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _state = device.state.lock().unwrap();
        panic!("poison wgpu render device state lock");
    }));
    assert!(poison.is_err());

    assert_eq!(
        device.transient_allocator_stats(),
        TransientAllocatorStats::default()
    );
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "poisoned-staging",
            4,
            BufferUsage::STAGING_READ | BufferUsage::COPY_DST,
        ))
        .expect("poisoned render device state lock should recover for creates");
    let ticket = device
        .write_buffer(buffer, 0, &[1, 2, 3, 4])
        .expect("poisoned render device state lock should recover for writes");
    device
        .flush_submissions()
        .expect("poisoned render device state lock should recover for flushes");
    device
        .poll_submissions()
        .expect("poisoned render device state lock should recover for polls");
    assert_eq!(
        device
            .submission_status(ticket)
            .expect("poisoned render device state lock should recover for tickets"),
        SubmissionStatus::Completed
    );
    assert_eq!(
        device
            .read_buffer(buffer, 0, 4)
            .expect("poisoned render device state lock should recover for reads"),
        vec![1, 2, 3, 4]
    );
}

#[test]
fn deterministic_buffer_batch_uses_one_ticket_for_all_ranges() {
    let device = DeterministicRhiContractDevice::new_headless();
    let first = device
        .create_buffer(&BufferDesc::new(
            "batch-first",
            4,
            BufferUsage::STAGING_READ | BufferUsage::COPY_DST,
        ))
        .expect("first batch buffer must be created");
    let second = device
        .create_buffer(&BufferDesc::new(
            "batch-second",
            4,
            BufferUsage::STAGING_READ | BufferUsage::COPY_DST,
        ))
        .expect("second batch buffer must be created");
    let payload: Arc<[u8]> = Arc::from([1_u8, 2, 3, 4, 5, 6, 7, 8]);
    let mut batch = BufferUploadBatch::new();
    batch.push(
        zr_rhi::BufferUpload::new(first, 0, Arc::clone(&payload), 0..4)
            .expect("first batch range must be valid"),
    );
    batch.push(
        zr_rhi::BufferUpload::new(second, 0, payload, 4..8)
            .expect("second batch range must be valid"),
    );

    let ticket = device
        .write_buffer_batch(batch)
        .expect("buffer batch must be accepted");
    assert_eq!(device.flush_submissions(), Ok(1));
    let poll = device
        .poll_submissions()
        .expect("deterministic completion poll must return a receipt");
    assert_eq!(poll.device_id(), device.device_id());
    assert_eq!(poll.generation(), device.generation());
    assert_eq!(poll.sequence(), 1);
    assert_eq!(
        device.submission_status(ticket),
        Ok(SubmissionStatus::Completed)
    );
    assert_eq!(device.read_buffer(first, 0, 4), Ok(vec![1, 2, 3, 4]));
    assert_eq!(device.read_buffer(second, 0, 4), Ok(vec![5, 6, 7, 8]));
}

#[test]
fn deterministic_submission_status_batch_preserves_order_and_per_ticket_failures() {
    let device = DeterministicRhiContractDevice::new_headless();
    let first = device
        .create_command_list(RenderQueueClass::Copy, "status-batch-first")
        .and_then(|list| device.enqueue_command_list(list))
        .expect("first status-batch ticket must be accepted");
    let second = device
        .create_command_list(RenderQueueClass::Copy, "status-batch-second")
        .and_then(|list| device.enqueue_command_list(list))
        .expect("second status-batch ticket must be accepted");
    let unknown = SubmissionTicket::new(
        device.device_id(),
        device.generation(),
        RenderQueueClass::Copy,
        second.sequence().saturating_add(100),
    );
    let mut statuses = Vec::new();

    device.append_submission_statuses(&[first, unknown, second], &mut statuses);

    assert_eq!(
        statuses,
        vec![
            Ok(SubmissionStatus::Accepted),
            Err(RhiError::UnknownSubmissionTicket(unknown)),
            Ok(SubmissionStatus::Accepted),
        ]
    );
}

#[test]
fn deterministic_poll_receipts_are_generation_qualified_and_strictly_monotonic() {
    let device = DeterministicRhiContractDevice::new_headless_with_identity(
        DeviceId::new(73),
        DeviceGeneration::new(9),
    );

    let first = device
        .poll_submissions()
        .expect("first deterministic poll receipt must be issued");
    let second = device
        .poll_submissions()
        .expect("second deterministic poll receipt must be issued");

    assert_eq!(first.device_id(), DeviceId::new(73));
    assert_eq!(first.generation(), DeviceGeneration::new(9));
    assert_eq!(first.sequence(), 1);
    assert_eq!(second.sequence(), 2);
}
