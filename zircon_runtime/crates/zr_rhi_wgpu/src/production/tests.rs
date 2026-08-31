use std::time::Duration;

use zr_rhi::{
    BufferDesc, BufferUsage, CommandList, DeviceFaultKind, DeviceGeneration, DeviceId,
    DiagnosticReadbackAdmission, DiagnosticReadbackBudget, DiagnosticReadbackTerminal,
    GpuMemoryBudget, GpuMemoryClass, RenderAdapterClass, RenderAdapterFacts, RenderBackendKind,
    RenderDevice, RenderDeviceFeatureSet, RenderDeviceProfile, RenderDeviceQueueTopology,
    RenderDeviceRequestPolicy, RenderQueueClass, RhiError, SubmissionLimits, SubmissionStatus,
};

use super::command_list::WgpuCommandList;
use super::device::{WgpuRenderDevice, WgpuRenderDeviceContext};
use super::registry::validate_wgpu_buffer_usage;
use crate::{next_wgpu_device_id, wgpu_adapter_facts, wgpu_device_limits, wgpu_device_request};

#[path = "tests/rendering.rs"]
mod rendering;

#[path = "tests/texture_copy.rs"]
mod texture_copy;

#[path = "tests/texture_views.rs"]
mod texture_views;

#[path = "tests/submission_packet.rs"]
mod submission_packet;

#[path = "tests/diagnostics.rs"]
mod diagnostics;

#[path = "tests/upload_batch.rs"]
mod upload_batch;

#[path = "tests/submission_metrics.rs"]
mod submission_metrics;

#[path = "tests/surface_bootstrap.rs"]
mod surface_bootstrap;

#[path = "tests/device_ownership.rs"]
mod device_ownership;

#[path = "tests/capabilities.rs"]
mod capabilities;

#[test]
fn production_device_rejects_a_context_with_a_different_adapter_profile() {
    let Some((context, _profile)) = production_test_context_with_policy(
        GpuMemoryBudget::reference_1080p_mid(),
        &RenderDeviceRequestPolicy::mvp_baseline(),
    ) else {
        return;
    };

    assert!(matches!(
        WgpuRenderDevice::new(context, test_profile()),
        Err(RhiError::NativeContextAdapterMismatch { .. })
    ));
}

#[test]
fn production_device_rejects_a_context_with_a_different_limits_profile() {
    let Some((context, profile)) = production_test_context_with_policy(
        GpuMemoryBudget::reference_1080p_mid(),
        &RenderDeviceRequestPolicy::mvp_baseline(),
    ) else {
        return;
    };
    let mut limits = profile.device_limits().clone();
    limits.max_bind_groups = if limits.max_bind_groups == 0 {
        1
    } else {
        limits.max_bind_groups - 1
    };
    let mismatched_profile = RenderDeviceProfile::new(
        profile.device_id(),
        profile.generation(),
        profile.adapter().clone(),
        RenderDeviceRequestPolicy::mvp_baseline()
            .negotiate(&profile.adapter().supported_features)
            .expect("the test adapter preserves the baseline negotiation"),
        limits,
        profile.queue_topology().clone(),
        profile.memory_budget(),
        profile.submission_limits(),
        profile.diagnostic_readback_budget(),
    );

    assert!(matches!(
        WgpuRenderDevice::new(context, mismatched_profile),
        Err(RhiError::NativeContextDeviceLimitsMismatch { .. })
    ));
}

#[test]
fn production_device_rejects_a_profile_with_non_wgpu_queue_topology() {
    let Some((context, profile)) = production_test_context_with_policy(
        GpuMemoryBudget::reference_1080p_mid(),
        &RenderDeviceRequestPolicy::mvp_baseline(),
    ) else {
        return;
    };
    let mut topology = profile.queue_topology().clone();
    topology.physical_queue_count = 2;
    topology.supports_async_compute = true;
    let mismatched_profile = RenderDeviceProfile::new(
        profile.device_id(),
        profile.generation(),
        profile.adapter().clone(),
        RenderDeviceRequestPolicy::mvp_baseline()
            .negotiate(&profile.adapter().supported_features)
            .expect("the test adapter preserves the baseline negotiation"),
        profile.device_limits().clone(),
        topology,
        profile.memory_budget(),
        profile.submission_limits(),
        profile.diagnostic_readback_budget(),
    );

    assert!(matches!(
        WgpuRenderDevice::new(context, mismatched_profile),
        Err(RhiError::NativeContextQueueTopologyMismatch { .. })
    ));
}

#[test]
fn production_device_rejects_profile_features_not_enabled_on_the_native_device() {
    let Some((context, profile)) = production_test_context_with_policy(
        GpuMemoryBudget::reference_1080p_mid(),
        &RenderDeviceRequestPolicy::mvp_baseline(),
    ) else {
        return;
    };
    let Some(feature) = profile.adapter().supported_features.iter().next() else {
        return;
    };
    let negotiation = RenderDeviceRequestPolicy::mvp_baseline()
        .with_optional_feature(feature)
        .negotiate(&profile.adapter().supported_features)
        .expect("the profile adapter reports the selected optional feature");
    assert!(negotiation.requested_features().contains(feature));
    let mismatched_profile = RenderDeviceProfile::new(
        profile.device_id(),
        profile.generation(),
        profile.adapter().clone(),
        negotiation,
        profile.device_limits().clone(),
        profile.queue_topology().clone(),
        profile.memory_budget(),
        profile.submission_limits(),
        profile.diagnostic_readback_budget(),
    );

    assert!(matches!(
        WgpuRenderDevice::new(context, mismatched_profile),
        Err(RhiError::NativeContextRequestedFeaturesMismatch { .. })
    ));
}

#[test]
fn native_wgpu_map_read_buffer_rejects_non_copy_destination_usage() {
    let desc = BufferDesc::new(
        "native-readback",
        256,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST | BufferUsage::STAGING_READ,
    );

    assert!(matches!(
        validate_wgpu_buffer_usage(&desc),
        Err(RhiError::InvalidBufferDescriptor { .. })
    ));
}

#[test]
fn native_wgpu_map_write_buffer_allows_only_copy_source_pairing() {
    let desc = BufferDesc::new(
        "native-upload",
        256,
        BufferUsage::STAGING_WRITE | BufferUsage::COPY_SRC,
    );

    assert!(validate_wgpu_buffer_usage(&desc).is_ok());
}

#[test]
fn production_memory_budget_rejects_resource_and_upload_admission_before_queue_work() {
    let budget = GpuMemoryBudget::new(16, 16, 4).with_max_pending_uploads(1);
    let Some(device) = production_test_device_with_budget(budget) else {
        return;
    };
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "production-budget-buffer",
            16,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();

    assert!(matches!(
        device.create_buffer(&BufferDesc::new(
            "production-budget-overflow",
            4,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        )),
        Err(RhiError::MemoryBudgetExceeded {
            class: GpuMemoryClass::Buffer,
            current_bytes: 16,
            requested_bytes: 4,
            limit_bytes: 16,
        })
    ));

    let first_upload = device.write_buffer(buffer, 0, &[1, 2, 3, 4]).unwrap();
    assert!(matches!(
        device.write_buffer(buffer, 4, &[5, 6, 7, 8]),
        Err(RhiError::UploadBackpressure {
            pending_uploads: 1,
            limit: 1,
        })
    ));
    assert_eq!(
        device.cancel_submission(first_upload).unwrap(),
        SubmissionStatus::Cancelled
    );
    let second_upload = device.write_buffer(buffer, 4, &[5, 6, 7, 8]).unwrap();
    assert_eq!(
        device.cancel_submission(second_upload).unwrap(),
        SubmissionStatus::Cancelled
    );
}

#[test]
fn production_memory_budget_rejects_staging_byte_overflow_before_queue_work() {
    let budget = GpuMemoryBudget::new(16, 16, 4).with_max_pending_uploads(2);
    let Some(device) = production_test_device_with_budget(budget) else {
        return;
    };
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "production-staging-budget-buffer",
            8,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();

    let first_upload = device.write_buffer(buffer, 0, &[1, 2, 3, 4]).unwrap();
    assert!(matches!(
        device.write_buffer(buffer, 4, &[5, 6, 7, 8]),
        Err(RhiError::MemoryBudgetExceeded {
            class: GpuMemoryClass::UploadStaging,
            current_bytes: 4,
            requested_bytes: 4,
            limit_bytes: 4,
        })
    ));
    assert_eq!(
        device.cancel_submission(first_upload).unwrap(),
        SubmissionStatus::Cancelled
    );
}

#[test]
fn production_command_list_keeps_neutral_queue_and_marker_data() {
    let mut command_list = WgpuCommandList::new(RenderQueueClass::Compute, "clear-history");
    command_list.push_debug_marker("prepare-dispatch");

    assert_eq!(command_list.queue_class(), RenderQueueClass::Compute);
    assert_eq!(command_list.label(), Some("clear-history"));
    assert_eq!(command_list.recorded_command_count(), 1);
}

#[test]
fn production_owner_is_usable_through_the_object_safe_rhi_contract() {
    let Some(device) = production_test_device() else {
        return;
    };
    let device: Box<dyn RenderDevice> = Box::new(device);
    let source = device
        .create_buffer(&BufferDesc::new(
            "object-safe-copy-source",
            16,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "object-safe-copy-destination",
            16,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    device
        .write_buffer(
            source,
            0,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        )
        .unwrap();
    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "object-safe-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(source, destination, 0, 0, 16);
    let ticket = device.submit(command_list).unwrap();
    wait_for_submission(device.as_ref(), ticket);
}

#[test]
fn production_submission_service_exposes_accepted_submitted_and_completed_lifecycle() {
    let Some(device) = production_test_device() else {
        return;
    };
    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "production-submission-lifecycle")
        .unwrap();
    let ticket = device.enqueue_command_list(command_list).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert!(matches!(
        device.wait_for_submission(ticket, Duration::ZERO),
        Err(RhiError::SubmissionWaitTimedOut {
            ticket: timed_out_ticket,
            timeout: Duration::ZERO,
        }) if timed_out_ticket == ticket
    ));
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, ticket);
}

#[test]
fn production_native_recording_is_enqueued_without_feature_owned_flush() {
    let Some(device) = production_test_device() else {
        return;
    };
    let mut recording = device
        .begin_native_recording(RenderQueueClass::Graphics)
        .expect("native recording lease");
    recording
        .record_command_buffer("production-native-recording", |_device, encoder| {
            encoder.insert_debug_marker("production-native-recording-marker");
            Ok::<(), RhiError>(())
        })
        .expect("native command buffer");
    let packet = recording.finish().expect("non-empty native packet");
    let ticket = device
        .enqueue_native_recording_packet(packet)
        .expect("owner-qualified native packet");

    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, ticket);
}

#[test]
fn production_submission_cancellation_retires_accepted_context_without_submitting() {
    let Some(device) = production_test_device() else {
        return;
    };
    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "production-cancel-before-submit")
        .unwrap();
    let ticket = device.enqueue_command_list(command_list).unwrap();
    assert_eq!(device.command_context_pool_counts_for_tests(), (1, 0));

    assert_eq!(
        device.cancel_submission(ticket).unwrap(),
        SubmissionStatus::Cancelled
    );
    assert_eq!(device.command_context_pool_counts_for_tests(), (1, 1));
    assert_eq!(device.flush_submissions().unwrap(), 0);
    assert_eq!(
        device.wait_for_submission(ticket, Duration::ZERO).unwrap(),
        SubmissionStatus::Cancelled
    );
}

#[test]
fn production_upload_receipt_is_cancellable_before_the_native_queue_observes_it() {
    let Some(device) = production_test_device() else {
        return;
    };
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "production-cancelled-upload",
            4,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let ticket = device.write_buffer(buffer, 0, &[9, 8, 7, 6]).unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(
        device.cancel_submission(ticket).unwrap(),
        SubmissionStatus::Cancelled
    );
    assert_eq!(device.flush_submissions().unwrap(), 0);
}

#[test]
fn production_diagnostic_readback_cancellation_terminalizes_the_submission_qualified_request() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_buffer(&BufferDesc::new(
            "production-diagnostic-cancel-source",
            4,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();

    device.begin_diagnostic_readback_frame(41).unwrap();
    let request = match device
        .enqueue_diagnostic_buffer_readback(source, 0, 4)
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("readback request unexpectedly rejected: {receipt:?}")
        }
    };
    let frame = device
        .submit_diagnostic_readback_frame("production-diagnostic-cancel")
        .unwrap()
        .expect("one admitted request must produce a submission-qualified frame");

    assert_eq!(
        device.cancel_submission(frame.submission()).unwrap(),
        SubmissionStatus::Cancelled
    );
    let delivery = device
        .take_diagnostic_readback_delivery()
        .expect("cancelled diagnostic request must become observable exactly once");
    assert_eq!(delivery.receipt().request(), request);
    assert_eq!(delivery.receipt().frame_key(), Some(frame));
    assert_eq!(
        delivery.receipt().terminal(),
        DiagnosticReadbackTerminal::Cancelled
    );
    assert_eq!(delivery.bytes(), None);
    assert!(device.take_diagnostic_readback_delivery().is_none());
}

#[test]
fn production_diagnostic_readback_copies_and_maps_through_the_submission_owner() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_buffer(&BufferDesc::new(
            "production-diagnostic-map-source",
            8,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let upload = device
        .write_buffer(source, 0, &[2, 3, 5, 7, 11, 13, 17, 19])
        .unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, upload);

    device.begin_diagnostic_readback_frame(42).unwrap();
    let request = match device
        .enqueue_diagnostic_buffer_readback(source, 0, 8)
        .unwrap()
    {
        DiagnosticReadbackAdmission::Admitted(request) => request,
        DiagnosticReadbackAdmission::Rejected(receipt) => {
            panic!("readback request unexpectedly rejected: {receipt:?}")
        }
    };
    let frame = device
        .submit_diagnostic_readback_frame("production-diagnostic-map")
        .unwrap()
        .expect("one admitted request must produce a submission-qualified frame");
    assert_eq!(device.flush_submissions().unwrap(), 1);

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let delivery = loop {
        device.poll_submissions().unwrap();
        if let Some(delivery) = device.take_diagnostic_readback_delivery() {
            break delivery;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "diagnostic map timed out"
        );
        std::thread::yield_now();
    };
    assert_eq!(delivery.receipt().request(), request);
    assert_eq!(delivery.receipt().frame_key(), Some(frame));
    assert_eq!(
        delivery.receipt().terminal(),
        DiagnosticReadbackTerminal::Succeeded
    );
    assert_eq!(delivery.bytes(), Some(&[2, 3, 5, 7, 11, 13, 17, 19][..]));
}

#[test]
fn production_upload_retains_a_destroyed_target_until_its_ticket_completes() {
    let Some(device) = production_test_device() else {
        return;
    };
    let buffer = device
        .create_buffer(&BufferDesc::new(
            "production-retired-upload-target",
            16,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let ticket = device.write_buffer(buffer, 0, &[0; 16]).unwrap();
    device.destroy_buffer(buffer).unwrap();
    assert!(device.buffer_desc(buffer).is_err());
    assert_eq!(device.transient_allocator_stats().bytes_reserved, 16);
    assert_eq!(
        device.memory_snapshot(),
        zr_rhi::GpuMemorySnapshot {
            retired_buffer_bytes: 16,
            pending_upload_bytes: 16,
            retired_allocations: 1,
            ..zr_rhi::GpuMemorySnapshot::default()
        }
    );

    assert_eq!(device.flush_submissions().unwrap(), 1);
    assert_eq!(device.memory_snapshot().pending_upload_bytes, 0);
    wait_for_submission(&device, ticket);
    device.poll_submissions().unwrap();
    assert_eq!(
        device.memory_snapshot(),
        zr_rhi::GpuMemorySnapshot::default()
    );
}

#[test]
fn production_submission_context_metadata_recycles_only_after_completion() {
    let Some(device) = production_test_device() else {
        return;
    };
    let first = device
        .create_command_list(RenderQueueClass::Copy, "production-context-first")
        .unwrap();
    let first_ticket = device.enqueue_command_list(first).unwrap();
    assert_eq!(device.command_context_pool_counts_for_tests(), (1, 0));

    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, first_ticket);
    assert_eq!(device.command_context_pool_counts_for_tests(), (1, 1));

    let second = device
        .create_command_list(RenderQueueClass::Copy, "production-context-second")
        .unwrap();
    let second_ticket = device.enqueue_command_list(second).unwrap();
    assert_eq!(device.command_context_pool_counts_for_tests(), (1, 0));
    assert_eq!(device.flush_submissions().unwrap(), 1);
    wait_for_submission(&device, second_ticket);
}

#[test]
fn production_submission_fault_terminalizes_accepted_ticket_without_a_device_wait() {
    let Some(device) = production_test_device() else {
        return;
    };
    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "production-fault-terminal")
        .unwrap();
    let ticket = device.enqueue_command_list(command_list).unwrap();

    device.inject_test_fault(DeviceFaultKind::DeviceDestroyed);
    device.poll_submissions().unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::DeviceLost
    );
    assert_eq!(
        device.wait_for_submission(ticket, Duration::ZERO).unwrap(),
        SubmissionStatus::DeviceLost
    );
}

#[test]
fn production_destroy_invalidates_the_handle_but_retains_native_buffer_until_completion() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_buffer(&BufferDesc::new(
            "retirement-source",
            16,
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "retirement-destination",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();
    let upload_ticket = device
        .write_buffer(source, 0, &[0; 16])
        .expect("retirement source upload must be valid");
    assert_eq!(
        device.submission_status(upload_ticket).unwrap(),
        SubmissionStatus::Accepted
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Copy, "retirement-copy")
        .unwrap();
    command_list.copy_buffer_to_buffer(source, destination, 0, 0, 16);
    let ticket = device.enqueue_command_list(command_list).unwrap();
    device.destroy_buffer(destination).unwrap();
    assert!(device.buffer_desc(destination).is_err());
    assert_eq!(device.transient_allocator_stats().bytes_reserved, 32);

    assert_eq!(device.flush_submissions().unwrap(), 2);
    wait_for_submission(&device, ticket);
    device.poll_submissions().unwrap();
    assert_eq!(device.transient_allocator_stats().bytes_reserved, 16);
}

#[test]
fn production_retirement_waits_for_earlier_submitted_use_after_later_packet_is_cancelled() {
    let Some(device) = production_test_device() else {
        return;
    };
    let source = device
        .create_buffer(&BufferDesc::new(
            "retirement-cancel-source",
            4,
            BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new(
            "retirement-cancel-destination",
            4,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    let mut first = device
        .create_command_list(RenderQueueClass::Copy, "retirement-first-submitted-use")
        .unwrap();
    first.copy_buffer_to_buffer(source, destination, 0, 0, 4);
    let first_ticket = device.enqueue_command_list(first).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    assert_eq!(
        device.submission_status(first_ticket).unwrap(),
        SubmissionStatus::Submitted
    );

    let mut later = device
        .create_command_list(RenderQueueClass::Copy, "retirement-later-cancelled-use")
        .unwrap();
    later.copy_buffer_to_buffer(source, destination, 0, 0, 4);
    let later_ticket = device.enqueue_command_list(later).unwrap();
    assert_eq!(
        device.cancel_submission(later_ticket).unwrap(),
        SubmissionStatus::Cancelled
    );

    device.destroy_buffer(destination).unwrap();
    assert_eq!(device.memory_snapshot().retired_buffer_bytes, 4);

    wait_for_submission(&device, first_ticket);
    device.poll_submissions().unwrap();
    assert_eq!(device.memory_snapshot().retired_buffer_bytes, 0);
}

#[test]
fn production_native_queue_submission_is_owned_only_by_the_submission_service() {
    let device_source = include_str!("device.rs");
    let service_source = include_str!("submission.rs");

    assert!(!device_source.contains(".queue.submit("));
    assert!(!device_source.contains(".queue.write_buffer("));
    assert!(!device_source.contains(".queue.write_texture("));
    assert!(service_source.contains("self.queue.submit(std::mem::take(command_buffers))"));
    assert!(service_source.contains("for upload in batch.into_uploads()"));
    assert!(service_source
        .contains(".write_buffer(upload.buffer(), upload.offset(), upload.payload())"));
    assert!(device_source.contains("commit_texture_upload"));
    assert!(service_source.contains("self.queue.write_texture("));
}

#[test]
fn production_diagnostic_readback_service_cannot_become_a_second_queue_or_poll_owner() {
    let device_source = include_str!("device.rs");
    let diagnostic_device_source = include_str!("device/diagnostics.rs");
    let diagnostics_source = include_str!("diagnostics/readback/service.rs");
    let layout_source = include_str!("diagnostics/readback/layout.rs");

    assert!(device_source.contains(".collect_completed_maps("));
    assert!(diagnostic_device_source.contains("enqueue_diagnostic_texture_readback"));
    assert!(diagnostic_device_source.contains("copy_texture_to_buffer("));
    assert!(diagnostics_source.contains("SubmissionTicket"));
    assert!(layout_source.contains("DiagnosticTextureReadbackLayout"));
    assert!(diagnostics_source.contains("map_async("));
    assert!(!diagnostics_source.contains("queue.submit("));
    assert!(!diagnostics_source.contains("queue.write_buffer("));
    assert!(!diagnostics_source.contains("device.poll("));
    assert!(!diagnostics_source.contains("wait_indefinitely"));
}

fn production_test_device() -> Option<WgpuRenderDevice> {
    production_test_device_with_policy(
        GpuMemoryBudget::reference_1080p_mid(),
        &RenderDeviceRequestPolicy::mvp_baseline(),
    )
}

fn production_test_device_with_budget(memory_budget: GpuMemoryBudget) -> Option<WgpuRenderDevice> {
    production_test_device_with_policy(memory_budget, &RenderDeviceRequestPolicy::mvp_baseline())
}

fn production_test_device_with_policy(
    memory_budget: GpuMemoryBudget,
    policy: &RenderDeviceRequestPolicy,
) -> Option<WgpuRenderDevice> {
    let (context, profile) = production_test_context_with_policy(memory_budget, policy)?;
    Some(
        WgpuRenderDevice::new(context, profile)
            .expect("the test context and its profile must describe the same adapter"),
    )
}

fn production_test_context_with_policy(
    memory_budget: GpuMemoryBudget,
    policy: &RenderDeviceRequestPolicy,
) -> Option<(WgpuRenderDeviceContext, RenderDeviceProfile)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let request = wgpu_device_request(adapter.features(), policy).ok()?;
    let adapter_facts = wgpu_adapter_facts(&adapter.get_info(), adapter.features());
    let (native_device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("zircon-production-rhi-test-device"),
            required_features: request.requested_features(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .ok()?;
    let profile = RenderDeviceProfile::new(
        next_wgpu_device_id(),
        DeviceGeneration::initial(),
        adapter_facts,
        request.feature_negotiation().clone(),
        wgpu_device_limits(&native_device.limits()),
        RenderDeviceQueueTopology::single_serialized_queue(),
        memory_budget,
        SubmissionLimits::default(),
        DiagnosticReadbackBudget::default(),
    );
    Some((
        WgpuRenderDeviceContext::new(instance, adapter, native_device, queue),
        profile,
    ))
}

fn test_profile() -> RenderDeviceProfile {
    let feature_negotiation = RenderDeviceRequestPolicy::mvp_baseline()
        .negotiate(&RenderDeviceFeatureSet::default())
        .expect("the empty MVP feature policy must negotiate");
    RenderDeviceProfile::new(
        DeviceId::new(1),
        DeviceGeneration::initial(),
        RenderAdapterFacts::new(
            RenderBackendKind::Vulkan,
            "production-capability-test",
            1,
            2,
            "test-driver",
            RenderAdapterClass::Other,
            None,
            RenderDeviceFeatureSet::default(),
        ),
        feature_negotiation,
        wgpu_device_limits(&wgpu::Limits::default()),
        RenderDeviceQueueTopology::single_serialized_queue(),
        GpuMemoryBudget::reference_1080p_mid(),
        SubmissionLimits::default(),
        DiagnosticReadbackBudget::default(),
    )
}

fn wait_for_submission(device: &dyn RenderDevice, ticket: zr_rhi::SubmissionTicket) {
    assert_eq!(
        device
            .wait_for_submission(ticket, Duration::from_secs(5))
            .unwrap(),
        SubmissionStatus::Completed
    );
}
