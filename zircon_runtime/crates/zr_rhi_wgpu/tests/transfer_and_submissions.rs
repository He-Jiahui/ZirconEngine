use std::time::Duration;

use super::*;
use zr_rhi::{GpuMemoryBudget, GpuMemoryClass, SubmissionLimits};

#[test]
fn deterministic_rhi_contract_enforces_resource_upload_and_submission_backpressure() {
    let device = DeterministicRhiContractDevice::new_headless_with_limits(
        GpuMemoryBudget::new(16, 16, 4).with_max_pending_uploads(1),
        SubmissionLimits::new(1, 2),
    );
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "budgeted-upload-target",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    assert_eq!(
        device
            .create_buffer(&BufferDesc::new(
                "buffer-over-budget",
                4,
                BufferUsage::COPY_DST,
            ))
            .unwrap_err(),
        zr_rhi::RhiError::MemoryBudgetExceeded {
            class: GpuMemoryClass::Buffer,
            current_bytes: 16,
            requested_bytes: 4,
            limit_bytes: 16,
        }
    );

    let first_upload = device.write_buffer(buffer, 0, &[1, 2, 3, 4]).unwrap();
    assert_eq!(
        device.write_buffer(buffer, 4, &[5]).unwrap_err(),
        zr_rhi::RhiError::UploadBackpressure {
            pending_uploads: 1,
            limit: 1,
        }
    );
    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "submission-over-budget")
        .unwrap();
    assert_eq!(
        device.enqueue_command_list(command_list).unwrap_err(),
        zr_rhi::RhiError::SubmissionBackpressure {
            unresolved_submissions: 1,
            limit: 1,
        }
    );

    assert_eq!(
        device.cancel_submission(first_upload).unwrap(),
        SubmissionStatus::Cancelled
    );
    let second_upload = device.write_buffer(buffer, 4, &[5]).unwrap();
    assert_eq!(
        device.cancel_submission(second_upload).unwrap(),
        SubmissionStatus::Cancelled
    );
}

#[test]
fn deterministic_rhi_contract_enforces_staging_byte_budget() {
    let device = DeterministicRhiContractDevice::new_headless_with_limits(
        GpuMemoryBudget::new(16, 16, 4).with_max_pending_uploads(2),
        SubmissionLimits::new(2, 2),
    );
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "staging-budget-target",
            8,
            BufferUsage::COPY_DST,
        ))
        .unwrap();
    let upload = device.write_buffer(buffer, 0, &[1, 2, 3, 4]).unwrap();

    assert_eq!(
        device.write_buffer(buffer, 4, &[5]).unwrap_err(),
        zr_rhi::RhiError::MemoryBudgetExceeded {
            class: GpuMemoryClass::UploadStaging,
            current_bytes: 4,
            requested_bytes: 1,
            limit_bytes: 4,
        }
    );
    assert_eq!(
        device.cancel_submission(upload).unwrap(),
        SubmissionStatus::Cancelled
    );
}

#[test]
fn deterministic_rhi_contract_submission_lifecycle_rejects_unissued_tickets() {
    let device = DeterministicRhiContractDevice::new_headless();
    let unknown_zero = zr_rhi::SubmissionTicket::new(
        zr_rhi::DeviceId::new(1),
        zr_rhi::DeviceGeneration::initial(),
        RenderQueueClass::Copy,
        0,
    );
    let unknown_future = zr_rhi::SubmissionTicket::new(
        zr_rhi::DeviceId::new(1),
        zr_rhi::DeviceGeneration::initial(),
        RenderQueueClass::Copy,
        7,
    );

    assert_eq!(
        device.submission_status(unknown_zero).unwrap_err(),
        zr_rhi::RhiError::UnknownSubmissionTicket(unknown_zero)
    );
    assert_eq!(
        device.submission_status(unknown_future).unwrap_err(),
        zr_rhi::RhiError::UnknownSubmissionTicket(unknown_future)
    );

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "empty-copy")
        .unwrap();
    let ticket = device.enqueue_command_list(command_list).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(device.flush_submissions().unwrap(), 1);
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Submitted
    );
    assert_eq!(
        device.cancel_submission(ticket).unwrap_err(),
        zr_rhi::RhiError::SubmissionCannotCancel {
            ticket,
            status: SubmissionStatus::Submitted,
        }
    );
    device.poll_submissions().unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Completed
    );
    assert_eq!(
        device.submission_status(unknown_future).unwrap_err(),
        zr_rhi::RhiError::UnknownSubmissionTicket(unknown_future)
    );
}

#[test]
fn deterministic_submission_cancellation_never_waits_for_the_device() {
    let device = DeterministicRhiContractDevice::new_headless();
    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "cancel-before-submit")
        .unwrap();
    let ticket = device.enqueue_command_list(command_list).unwrap();

    assert_eq!(
        device.cancel_submission(ticket).unwrap(),
        SubmissionStatus::Cancelled
    );
    assert_eq!(device.flush_submissions().unwrap(), 0);
    assert_eq!(
        device.wait_for_submission(ticket, Duration::ZERO).unwrap(),
        SubmissionStatus::Cancelled
    );
}

#[test]
fn deterministic_upload_receipt_is_observable_and_cancellable_before_flush() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "cancelled-upload-source",
            4,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "cancelled-upload-destination",
            4,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();

    let upload_ticket = device.write_buffer(upload, 0, &[9, 8, 7, 6]).unwrap();
    assert_eq!(
        device.submission_status(upload_ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(device.memory_snapshot().pending_upload_bytes, 4);
    assert_eq!(
        device.cancel_submission(upload_ticket).unwrap(),
        SubmissionStatus::Cancelled
    );
    assert_eq!(device.memory_snapshot().pending_upload_bytes, 0);

    let mut copy = device
        .create_command_list(RenderQueueClass::Copy, "copy-after-cancelled-upload")
        .unwrap();
    copy.copy_buffer_to_buffer(upload, destination, 0, 0, 4);
    device.submit(copy).unwrap();
    assert_eq!(device.read_buffer(destination, 0, 4).unwrap(), vec![0; 4]);
}

#[test]
fn deterministic_rhi_contract_write_copy_and_read_buffer_preserves_bytes() {
    let device = DeterministicRhiContractDevice::new_headless();
    let upload = device
        .create_buffer(&BufferDesc::new(
            "upload",
            16,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let gpu_buffer = device
        .create_buffer(&BufferDesc::new(
            "gpu-buffer",
            16,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();

    let upload_ticket = device
        .write_buffer(upload, 4, &[10, 20, 30, 40, 50, 60])
        .unwrap();
    assert_eq!(
        device.submission_status(upload_ticket).unwrap(),
        SubmissionStatus::Accepted
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "upload-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(upload, gpu_buffer, 4, 2, 6);
    let ticket = device.submit(command_list).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Completed
    );

    assert_eq!(
        device.read_buffer(gpu_buffer, 0, 10).unwrap(),
        vec![0, 0, 10, 20, 30, 40, 50, 60, 0, 0]
    );
}

#[test]
fn deterministic_texture_upload_is_ticketed_and_discards_terminal_row_padding() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(&TextureDesc::new(
            "texture-upload",
            4,
            3,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let region = TextureCopyRegion::new(2, 2).with_origin(1, 1, 0);
    let source = [
        1, 2, 3, 4, 5, 6, 7, 8, 90, 91, 92, 93, 9, 10, 11, 12, 13, 14, 15, 16, 94, 95, 96, 97,
    ];

    let ticket = device.write_texture(texture, region, 12, &source).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(device.memory_snapshot().pending_upload_bytes, 20);
    assert_eq!(device.flush_submissions().unwrap(), 1);
    device.poll_submissions().unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Completed
    );

    let mut expected = vec![0; 4 * 3 * 4];
    expected[20..28].copy_from_slice(&source[..8]);
    expected[36..44].copy_from_slice(&source[12..20]);
    assert_eq!(device.read_texture(texture).unwrap(), expected);
}

#[test]
fn deterministic_texture_upload_rejects_non_color_or_undersized_sources() {
    let device = DeterministicRhiContractDevice::new_headless();
    let texture = device
        .create_texture(&TextureDesc::new(
            "texture-upload-validation",
            2,
            2,
            TextureFormat::Rgba8Unorm,
            TextureUsage::COPY_DST,
        ))
        .unwrap();

    assert!(matches!(
        device.write_texture(texture, TextureCopyRegion::new(2, 2), 7, &[0; 16]),
        Err(zr_rhi::RhiError::TextureWriteOutOfRange { .. })
    ));
    assert!(matches!(
        device.write_texture(texture, TextureCopyRegion::new(2, 2), 8, &[0; 15]),
        Err(zr_rhi::RhiError::TextureWriteOutOfRange { .. })
    ));

    let depth = device
        .create_texture(&TextureDesc::new(
            "depth-texture-upload-validation",
            1,
            1,
            TextureFormat::Depth24PlusStencil8,
            TextureUsage::COPY_DST,
        ))
        .unwrap();
    assert!(matches!(
        device.write_texture(depth, TextureCopyRegion::new(1, 1), 4, &[0; 4]),
        Err(zr_rhi::RhiError::InvalidCopy { .. })
    ));
}

#[test]
fn deterministic_submission_batch_preserves_packet_order() {
    let device = DeterministicRhiContractDevice::new_headless();
    let source = device
        .create_buffer(&BufferDesc::new(
            "ordered-source",
            4,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let intermediate = device
        .create_buffer(&BufferDesc::new(
            "ordered-intermediate",
            4,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "ordered-destination",
            4,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();
    let upload_ticket = device.write_buffer(source, 0, &[9, 8, 7, 6]).unwrap();

    let mut first = device
        .create_command_list(RenderQueueClass::Copy, "ordered-first")
        .unwrap();
    first.copy_buffer_to_buffer(source, intermediate, 0, 0, 4);
    let first_ticket = device.enqueue_command_list(first).unwrap();

    let mut second = device
        .create_command_list(RenderQueueClass::Copy, "ordered-second")
        .unwrap();
    second.copy_buffer_to_buffer(intermediate, destination, 0, 0, 4);
    let second_ticket = device.enqueue_command_list(second).unwrap();

    assert_eq!(device.flush_submissions().unwrap(), 3);
    device.poll_submissions().unwrap();
    assert_eq!(
        device.submission_status(upload_ticket).unwrap(),
        SubmissionStatus::Completed
    );
    assert_eq!(
        device.submission_status(first_ticket).unwrap(),
        SubmissionStatus::Completed
    );
    assert_eq!(
        device.submission_status(second_ticket).unwrap(),
        SubmissionStatus::Completed
    );
    assert_eq!(
        device.read_buffer(destination, 0, 4).unwrap(),
        vec![9, 8, 7, 6]
    );
}

#[test]
fn deterministic_submission_packet_uses_one_ticket_and_preserves_list_order() {
    let device = DeterministicRhiContractDevice::new_headless();
    let source = device
        .create_buffer(&BufferDesc::new(
            "packet-source",
            4,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let intermediate = device
        .create_buffer(&BufferDesc::new(
            "packet-intermediate",
            4,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "packet-destination",
            4,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();

    let upload = device.write_buffer(source, 0, &[7, 6, 5, 4]).unwrap();
    let mut first = device
        .create_command_list(RenderQueueClass::Copy, "packet-first-copy")
        .unwrap();
    first.copy_buffer_to_buffer(source, intermediate, 0, 0, 4);
    let mut second = device
        .create_command_list(RenderQueueClass::Copy, "packet-second-copy")
        .unwrap();
    second.copy_buffer_to_buffer(intermediate, destination, 0, 0, 4);

    let packet = device
        .create_submission_packet(RenderQueueClass::Copy, vec![first, second])
        .unwrap();
    assert_eq!(packet.command_list_count(), 2);
    let packet_ticket = device.enqueue_submission_packet(packet).unwrap();
    assert_eq!(
        device.submission_status(packet_ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(device.flush_submissions().unwrap(), 2);
    device.poll_submissions().unwrap();

    assert_eq!(
        device.submission_status(upload).unwrap(),
        SubmissionStatus::Completed
    );
    assert_eq!(
        device.submission_status(packet_ticket).unwrap(),
        SubmissionStatus::Completed
    );
    assert_eq!(
        device.read_buffer(destination, 0, 4).unwrap(),
        vec![7, 6, 5, 4]
    );
}

#[test]
fn deterministic_submission_interleaves_uploads_and_commands_in_ticket_order() {
    let device = DeterministicRhiContractDevice::new_headless();
    let source = device
        .create_buffer(&BufferDesc::new(
            "interleaved-source",
            4,
            BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "interleaved-destination",
            8,
            BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
        ))
        .unwrap();

    let first_upload = device.write_buffer(source, 0, &[1, 0, 0, 0]).unwrap();
    let mut first_copy = device
        .create_command_list(RenderQueueClass::Copy, "interleaved-first-copy")
        .unwrap();
    first_copy.copy_buffer_to_buffer(source, destination, 0, 0, 4);
    let first_copy_ticket = device.enqueue_command_list(first_copy).unwrap();

    let second_upload = device.write_buffer(source, 0, &[2, 0, 0, 0]).unwrap();
    let mut second_copy = device
        .create_command_list(RenderQueueClass::Copy, "interleaved-second-copy")
        .unwrap();
    second_copy.copy_buffer_to_buffer(source, destination, 0, 4, 4);
    let second_copy_ticket = device.enqueue_command_list(second_copy).unwrap();

    assert_eq!(device.flush_submissions().unwrap(), 4);
    device.poll_submissions().unwrap();
    for ticket in [
        first_upload,
        first_copy_ticket,
        second_upload,
        second_copy_ticket,
    ] {
        assert_eq!(
            device.submission_status(ticket).unwrap(),
            SubmissionStatus::Completed
        );
    }
    assert_eq!(
        device.read_buffer(destination, 0, 8).unwrap(),
        vec![1, 0, 0, 0, 2, 0, 0, 0]
    );
}

#[test]
fn deterministic_rhi_contract_overlapping_self_copy_preserves_memmove_semantics() {
    let device = DeterministicRhiContractDevice::new_headless();
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "self-copy",
            8,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let readback = device
        .create_buffer(&BufferDesc::new(
            "self-copy-readback",
            8,
            BufferUsage::STAGING_READ | BufferUsage::COPY_DST,
        ))
        .unwrap();
    device
        .write_buffer(buffer, 0, &[1, 2, 3, 4, 5, 6, 7, 8])
        .unwrap();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "overlapping-self-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(buffer, buffer, 0, 2, 6);
    command_list.copy_buffer_to_buffer(buffer, readback, 0, 0, 8);
    device.submit(command_list).unwrap();

    assert_eq!(
        device.read_buffer(readback, 0, 8).unwrap(),
        vec![1, 2, 1, 2, 3, 4, 5, 6]
    );
}

#[test]
fn deterministic_rhi_contract_write_buffer_validates_usage_and_range() {
    let device = DeterministicRhiContractDevice::new_headless();
    let read_only = device
        .create_buffer(&BufferDesc::new("read-only", 8, BufferUsage::STAGING_READ))
        .unwrap();

    assert_eq!(
        device.write_buffer(read_only, 0, &[1, 2, 3]).unwrap_err(),
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: read_only.diagnostic_id(),
            required: BufferUsage::COPY_DST,
            actual: BufferUsage::STAGING_READ,
        }
    );

    let upload = device
        .create_buffer(&BufferDesc::new("upload", 8, BufferUsage::COPY_DST))
        .unwrap();
    assert_eq!(
        device.write_buffer(upload, 6, &[1, 2, 3]).unwrap_err(),
        zr_rhi::RhiError::WriteOutOfRange {
            buffer: upload.diagnostic_id(),
            offset: 6,
            size: 3,
        }
    );

    let mapped_write = device
        .create_buffer(&BufferDesc::new(
            "mapped-write",
            8,
            BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        ))
        .unwrap();
    assert_eq!(
        device
            .write_buffer(mapped_write, 0, &[1, 2, 3])
            .unwrap_err(),
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: mapped_write.diagnostic_id(),
            required: BufferUsage::COPY_DST,
            actual: BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
        }
    );
}

#[test]
fn deterministic_rhi_contract_read_texture_validates_usage() {
    let device = DeterministicRhiContractDevice::new_headless();
    let write_only = device
        .create_texture(&TextureDesc::new(
            "write-only-texture",
            2,
            2,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::COPY_DST,
        ))
        .unwrap();

    assert_eq!(
        device.read_texture(write_only).unwrap_err(),
        zr_rhi::RhiError::InvalidTextureUsage {
            texture: write_only.diagnostic_id(),
            required: TextureUsage::COPY_SRC,
            actual: TextureUsage::COPY_DST,
        }
    );
}

#[test]
fn deterministic_rhi_contract_read_buffer_validates_usage_and_range() {
    let device = DeterministicRhiContractDevice::new_headless();
    let non_readback = device
        .create_buffer(&BufferDesc::new("non-readback", 8, BufferUsage::COPY_DST))
        .unwrap();

    assert_eq!(
        device.read_buffer(non_readback, 0, 4).unwrap_err(),
        zr_rhi::RhiError::InvalidBufferUsage {
            buffer: non_readback.diagnostic_id(),
            required: BufferUsage::STAGING_READ,
            actual: BufferUsage::COPY_DST,
        }
    );

    let readback = device
        .create_buffer(&BufferDesc::new("readback", 8, BufferUsage::STAGING_READ))
        .unwrap();
    assert_eq!(
        device.read_buffer(readback, 6, 3).unwrap_err(),
        zr_rhi::RhiError::ReadbackOutOfRange {
            buffer: readback.diagnostic_id(),
            offset: 6,
            size: 3,
        }
    );
}
