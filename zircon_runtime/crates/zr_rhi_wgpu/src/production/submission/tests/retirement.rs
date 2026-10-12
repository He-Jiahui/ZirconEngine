//! Real native backing destructors must be able to enter the original coordinator.
//! These tests require a native adapter; unavailable native setup is a failure.
use std::sync::{mpsc, Arc, Weak};

use super::*;
use zr_rhi::{
    RenderDeviceRequestPolicy, RhiGraphAccessId, RhiGraphAccessRange, RhiGraphExecutionAccess,
    RhiGraphExecutionPass, RhiGraphPhysicalResourceLease, RhiGraphQueueLane,
    RhiGraphResourceAccessKind, RhiGraphResourceBounds, RhiGraphResourceId, RhiGraphResourceKind,
    RhiGraphResourceState,
};

fn native_service() -> (wgpu::Device, Arc<WgpuSubmissionService>) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("coordinator destructor regression requires a real native adapter");
    let request = crate::wgpu_device_request(
        adapter.features(),
        &RenderDeviceRequestPolicy::mvp_baseline(),
    )
    .unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("coordinator-retirement-regression"),
        required_features: request.requested_features(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .expect("coordinator destructor regression requires a real native device");
    let service = Arc::new(WgpuSubmissionService::new(
        queue,
        crate::next_wgpu_device_id(),
        DeviceGeneration::initial(),
        GpuMemoryBudget::reference_1080p_mid(),
        SubmissionLimits::default(),
    ));
    (device, service)
}

// The canonical receipt owns the only native buffer alias. This owner observes
// both actual service mutexes and then reenters actual cancel/flush, without
// waiting forever on the unfixed source. The channel records destructor return.
struct ReentrantNativeBuffer {
    buffer: Option<wgpu::Buffer>,
    service: Weak<WgpuSubmissionService>,
    ticket: SubmissionTicket,
    expected_status: SubmissionStatus,
    destroyed: mpsc::Sender<()>,
}

impl Drop for ReentrantNativeBuffer {
    fn drop(&mut self) {
        let service = self
            .service
            .upgrade()
            .expect("original service still owned");
        assert!(
            service.queue_access.try_lock().is_ok(),
            "native backing Drop held queue guard"
        );
        assert!(
            service.state.try_lock().is_ok(),
            "native backing Drop held state guard"
        );
        assert_eq!(service.cancel(self.ticket).unwrap(), self.expected_status);
        assert_eq!(service.flush().unwrap(), 0);
        let buffer = self.buffer.take().unwrap();
        assert_eq!(buffer.size(), 64);
        drop(buffer);
        self.destroyed.send(()).unwrap();
    }
}

fn accepted_packet(
    device: &wgpu::Device,
    service: &Arc<WgpuSubmissionService>,
    expected_status: SubmissionStatus,
) -> (
    SubmissionTicket,
    Weak<ReentrantNativeBuffer>,
    mpsc::Receiver<()>,
) {
    let ticket = service.begin_packet(RenderQueueClass::Graphics).unwrap();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("original-canonical-physical-backing"),
        size: 64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let (destroyed, observation) = mpsc::channel();
    let owner = Arc::new(ReentrantNativeBuffer {
        buffer: Some(buffer),
        service: Arc::downgrade(service),
        ticket,
        expected_status,
        destroyed,
    });
    let weak = Arc::downgrade(&owner);
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::Buffer, 0, 7);
    let access_id = RhiGraphAccessId::new(0, 7, 0);
    let access = RhiGraphExecutionAccess::new(
        access_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Read,
        RhiGraphAccessRange::buffer(0, 64),
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::Graphics,
        None,
        true,
    );
    let receipt = RhiGraphExecutionReceipt::new(
        service.device_id,
        service.generation,
        1,
        7,
        ticket.queue_class(),
        vec![RhiGraphExecutionPass::new(
            0,
            0,
            7,
            RhiGraphQueueLane::Graphics,
            vec![access],
            vec![],
        )],
        vec![RhiGraphPhysicalResourceLease::new(
            access_id,
            resource,
            None,
            service.device_id,
            service.generation,
            RhiGraphResourceBounds::buffer(64).unwrap(),
            owner,
        )],
    )
    .unwrap();
    let command = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("accepted-original-canonical-receipt"),
        })
        .finish();
    service
        .commit_packet_with_ui_image_pins_and_graph_receipt(ticket, vec![command], None, receipt)
        .unwrap();
    assert!(weak.upgrade().is_some());
    assert!(matches!(
        observation.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    (ticket, weak, observation)
}

fn assert_destroyed(weak: &Weak<ReentrantNativeBuffer>, observation: &mpsc::Receiver<()>) {
    observation
        .try_recv()
        .expect("real buffer destructor returned before cleanup result");
    assert!(weak.upgrade().is_none());
    assert!(matches!(
        observation.try_recv(),
        Err(mpsc::TryRecvError::Disconnected)
    ));
}

#[test]
fn native_receipt_cancel_destructor_reenters_original_service_after_both_unlocks() {
    let (device, service) = native_service();
    let (ticket, weak, observation) =
        accepted_packet(&device, &service, SubmissionStatus::Cancelled);
    assert_eq!(service.cancel(ticket).unwrap(), SubmissionStatus::Cancelled);
    assert_destroyed(&weak, &observation);
    assert_eq!(service.cancel(ticket).unwrap(), SubmissionStatus::Cancelled);
    assert_eq!(service.command_context_pool_counts(), (1, 1));
}

#[test]
fn native_receipt_abandoned_batch_preserves_order_duplicates_and_preflight_refusal() {
    let (device, service) = native_service();
    let (ticket, weak, observation) =
        accepted_packet(&device, &service, SubmissionStatus::Cancelled);
    let foreign = SubmissionTicket::new(
        crate::next_wgpu_device_id(),
        service.generation,
        ticket.queue_class(),
        ticket.sequence(),
    );
    assert!(
        matches!(service.settle_abandoned_submissions(&[ticket, foreign]), Err(RhiError::UnknownSubmissionTicket(value)) if value == foreign)
    );
    assert_eq!(
        service.lock_state().history.status(ticket),
        Some(SubmissionStatus::Accepted)
    );
    assert!(weak.upgrade().is_some());
    assert!(matches!(
        observation.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    assert_eq!(
        service
            .settle_abandoned_submissions(&[ticket, ticket])
            .unwrap(),
        vec![SubmissionStatus::Cancelled, SubmissionStatus::Cancelled]
    );
    assert_destroyed(&weak, &observation);
    assert_eq!(
        service.settle_abandoned_submissions(&[ticket]).unwrap(),
        vec![SubmissionStatus::Cancelled]
    );
}

#[test]
fn native_receipt_abandoned_batch_invariant_refusal_drops_after_both_unlocks() {
    let (device, service) = native_service();
    let (ticket, weak, observation) =
        accepted_packet(&device, &service, SubmissionStatus::Cancelled);
    let missing = service.begin_packet(RenderQueueClass::Graphics).unwrap();
    // Exercise the existing lowest-provider invariant refusal AFTER another
    // genuine accepted packet has been cancelled, without weakening the branch.
    assert!(service.lock_state().reserved.remove(&missing));
    assert!(
        matches!(service.settle_abandoned_submissions(&[ticket, missing]), Err(RhiError::UnknownSubmissionTicket(value)) if value == missing)
    );
    assert_destroyed(&weak, &observation);
    assert_eq!(
        service.lock_state().history.status(ticket),
        Some(SubmissionStatus::Cancelled)
    );
    assert_eq!(
        service.lock_state().history.status(missing),
        Some(SubmissionStatus::Accepted)
    );
    service.terminalize_unresolved(SubmissionStatus::Failed);
    // The deliberately missing pending/reserved entry retains the original
    // context-accounting refusal semantics; this fix does not repair corruption.
    assert_eq!(service.command_context_pool_counts(), (2, 1));
}

#[test]
fn native_receipt_fault_terminalization_destructor_reenters_after_both_unlocks() {
    let (device, service) = native_service();
    let (ticket, weak, observation) =
        accepted_packet(&device, &service, SubmissionStatus::DeviceLost);
    service.terminalize_unresolved(SubmissionStatus::DeviceLost);
    assert_destroyed(&weak, &observation);
    assert_eq!(
        service.cancel(ticket).unwrap(),
        SubmissionStatus::DeviceLost
    );
    service.terminalize_unresolved(SubmissionStatus::Failed);
    assert_eq!(
        service.cancel(ticket).unwrap(),
        SubmissionStatus::DeviceLost
    );
    assert_eq!(service.command_context_pool_counts(), (1, 1));
}

#[test]
fn native_submitted_cancel_refusal_and_abandoned_batch_keep_original_receipt_owned() {
    let (device, service) = native_service();
    let (submitted, submitted_weak, submitted_observation) =
        accepted_packet(&device, &service, SubmissionStatus::Failed);
    assert_eq!(service.flush().unwrap(), 1);
    assert!(
        matches!(service.cancel(submitted), Err(RhiError::SubmissionCannotCancel {
        ticket, status: SubmissionStatus::Submitted,
    }) if ticket == submitted)
    );
    assert!(submitted_weak.upgrade().is_some());
    assert!(matches!(
        submitted_observation.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    let (accepted, accepted_weak, accepted_observation) =
        accepted_packet(&device, &service, SubmissionStatus::Cancelled);
    assert_eq!(
        service
            .settle_abandoned_submissions(&[submitted, accepted])
            .unwrap(),
        vec![SubmissionStatus::Submitted, SubmissionStatus::Cancelled]
    );
    assert_destroyed(&accepted_weak, &accepted_observation);
    assert!(submitted_weak.upgrade().is_some());
    service.terminalize_unresolved(SubmissionStatus::Failed);
    assert_destroyed(&submitted_weak, &submitted_observation);
    assert_eq!(service.command_context_pool_counts(), (2, 2));
}
