use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};

use super::*;
use zr_rhi::{
    PresentMode, RenderClearColor, RenderNativeSurfaceTarget, RenderPassColorAttachmentDesc,
    RenderPassColorLoadOp, RenderPassStoreOp, RhiGraphAccessId, RhiGraphAccessRange,
    RhiGraphExecutionAccess, RhiGraphExecutionPass, RhiGraphPhysicalResourceLease,
    RhiGraphQueueLane, RhiGraphResourceAccessKind, RhiGraphResourceBounds, RhiGraphResourceId,
    RhiGraphResourceKind, RhiGraphResourceState, TextureFormat,
};

// The erased canonical lease owns actual bytes allocated by create_buffer and the
// original handle allocation. Weak observations do not keep either allocation alive.
struct PhysicalBufferLease {
    resource: Option<WgpuBufferResource>,
    handle: BufferHandle,
    allocator: RenderResourceHandleAllocator,
    state: Weak<Mutex<DeterministicRhiContractDeviceState>>,
    releases: Arc<AtomicUsize>,
}

impl Drop for PhysicalBufferLease {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            assert!(
                state.try_lock().is_ok(),
                "physical lease dropped under device state lock"
            );
        }
        let resource = self.resource.take().unwrap();
        assert_eq!(resource.contents.len(), 64);
        drop(resource);
        self.allocator.release_buffer(self.handle).unwrap();
        assert_eq!(self.releases.fetch_add(1, Ordering::SeqCst), 0);
    }
}

fn receipt(
    device: &DeterministicRhiContractDevice,
    queue: RenderQueueClass,
) -> (
    RhiGraphExecutionReceipt,
    Weak<PhysicalBufferLease>,
    Arc<AtomicUsize>,
) {
    let handle = device
        .create_buffer(&BufferDesc::new(
            "graph-owned-backing",
            64,
            BufferUsage::COPY_SRC,
        ))
        .unwrap();
    let resource = device.lock_state().buffers.remove(&handle).unwrap();
    let releases = Arc::new(AtomicUsize::new(0));
    let owner = Arc::new(PhysicalBufferLease {
        resource: Some(resource),
        handle,
        allocator: device.handle_allocator.clone(),
        state: Arc::downgrade(&device.state),
        releases: releases.clone(),
    });
    let weak = Arc::downgrade(&owner);
    let graph_resource = RhiGraphResourceId::new(RhiGraphResourceKind::Buffer, 0, 7);
    let access_id = RhiGraphAccessId::new(0, 7, 0);
    let access = RhiGraphExecutionAccess::new(
        access_id,
        graph_resource,
        1,
        RhiGraphResourceAccessKind::Read,
        RhiGraphAccessRange::buffer(0, 64),
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::Graphics,
        Some(handle.diagnostic_id()),
        true,
    );
    let receipt = RhiGraphExecutionReceipt::new(
        device.device_id(),
        device.generation(),
        1,
        7,
        queue,
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
            graph_resource,
            Some(handle.diagnostic_id()),
            device.device_id(),
            device.generation(),
            RhiGraphResourceBounds::buffer(64).unwrap(),
            owner,
        )],
    )
    .unwrap();
    (receipt, weak, releases)
}

fn packet(
    device: &DeterministicRhiContractDevice,
) -> (
    RhiSubmissionPacket,
    Weak<PhysicalBufferLease>,
    Arc<AtomicUsize>,
) {
    let queue = RenderQueueClass::Graphics;
    let command_list = device.create_command_list(queue, "graph-custody").unwrap();
    let packet = device
        .create_submission_packet(queue, vec![command_list])
        .unwrap();
    let (receipt, weak, releases) = receipt(device, queue);
    (
        packet.with_graph_execution_receipt(receipt).unwrap(),
        weak,
        releases,
    )
}

fn assert_live(weak: &Weak<PhysicalBufferLease>, releases: &AtomicUsize) {
    assert_eq!(
        weak.upgrade()
            .unwrap()
            .resource
            .as_ref()
            .unwrap()
            .contents
            .len(),
        64
    );
    assert_eq!(releases.load(Ordering::SeqCst), 0);
}

fn assert_released(weak: &Weak<PhysicalBufferLease>, releases: &AtomicUsize) {
    assert!(weak.upgrade().is_none());
    assert_eq!(releases.load(Ordering::SeqCst), 1);
}

#[test]
fn original_graph_physical_lease_survives_enqueue_and_submitted_cancel_refusal_until_completion() {
    let device = DeterministicRhiContractDevice::new_headless();
    let (packet, weak, releases) = packet(&device);
    let ticket = device.enqueue_submission_packet(packet).unwrap();
    assert_live(&weak, &releases);
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Accepted
    );
    assert_eq!(device.flush_submissions().unwrap(), 1);
    assert_live(&weak, &releases);
    assert!(matches!(
        device.cancel_submission(ticket),
        Err(RhiError::SubmissionCannotCancel {
            status: SubmissionStatus::Submitted,
            ..
        })
    ));
    assert_live(&weak, &releases);
    device.poll_submissions().unwrap();
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Completed
    );
    assert_released(&weak, &releases);
    device.poll_submissions().unwrap();
    assert_eq!(
        device.cancel_submission(ticket).unwrap(),
        SubmissionStatus::Completed
    );
    assert_released(&weak, &releases);
}

#[test]
fn original_graph_physical_lease_releases_once_after_accepted_cancellation() {
    let device = DeterministicRhiContractDevice::new_headless();
    let (packet, weak, releases) = packet(&device);
    let ticket = device.enqueue_submission_packet(packet).unwrap();
    assert_live(&weak, &releases);
    assert_eq!(
        device.cancel_submission(ticket).unwrap(),
        SubmissionStatus::Cancelled
    );
    assert_released(&weak, &releases);
    assert_eq!(device.flush_submissions().unwrap(), 0);
    assert_eq!(
        device.cancel_submission(ticket).unwrap(),
        SubmissionStatus::Cancelled
    );
    assert_released(&weak, &releases);
}

#[test]
fn original_graph_physical_lease_releases_on_foreign_device_preflight_refusal() {
    let owner = DeterministicRhiContractDevice::new_headless();
    let other = DeterministicRhiContractDevice::new_headless_with_identity(
        DeviceId::new(2),
        DeviceGeneration::initial(),
    );
    let (packet, weak, releases) = packet(&owner);
    assert!(matches!(
        other.enqueue_submission_packet(packet),
        Err(RhiError::SubmissionPacketDeviceMismatch { .. })
    ));
    assert_released(&weak, &releases);
    assert_eq!(owner.flush_submissions().unwrap(), 0);
    assert_eq!(other.flush_submissions().unwrap(), 0);
}

#[test]
fn original_graph_physical_lease_releases_after_real_execution_failure() {
    let device = DeterministicRhiContractDevice::new_headless();
    let source = device
        .create_buffer(&BufferDesc::new("source", 4, BufferUsage::COPY_SRC))
        .unwrap();
    let destination = device
        .create_buffer(&BufferDesc::new("destination", 4, BufferUsage::COPY_DST))
        .unwrap();
    let mut commands = device
        .create_command_list(RenderQueueClass::Copy, "graph-failing-copy")
        .unwrap();
    commands.copy_buffer_to_buffer(source, destination, 0, 0, 4);
    let packet = device
        .create_submission_packet(RenderQueueClass::Copy, vec![commands])
        .unwrap();
    let (receipt, weak, releases) = receipt(&device, RenderQueueClass::Copy);
    let ticket = device
        .enqueue_submission_packet(packet.with_graph_execution_receipt(receipt).unwrap())
        .unwrap();
    device.destroy_buffer(destination).unwrap();
    assert_live(&weak, &releases);
    assert_eq!(
        device.flush_submissions().unwrap_err(),
        RhiError::UnknownBuffer(destination.diagnostic_id())
    );
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Failed
    );
    assert_released(&weak, &releases);
    device.poll_submissions().unwrap();
    assert_released(&weak, &releases);
}

#[test]
fn original_graph_physical_lease_waits_for_last_device_owner_drop() {
    for submitted in [false, true] {
        let device = DeterministicRhiContractDevice::new_headless();
        let other_owner = device.clone();
        let (packet, weak, releases) = packet(&device);
        device.enqueue_submission_packet(packet).unwrap();
        if submitted {
            device.flush_submissions().unwrap();
        }
        drop(device);
        assert_live(&weak, &releases);
        drop(other_owner);
        assert_released(&weak, &releases);
    }
}

#[test]
fn original_graph_physical_lease_releases_unlocked_after_surface_cancellation() {
    for terminal in 0..3 {
        let device = DeterministicRhiContractDevice::new_headless();
        let descriptor = RenderSurfaceDescriptor::new(
            "graph-surface",
            RenderNativeSurfaceTarget::Win32 {
                hwnd: 1,
                hinstance: None,
            },
            SwapchainDesc {
                width: 4,
                height: 4,
                present_mode: PresentMode::Fifo,
                format: TextureFormat::Bgra8UnormSrgb,
            },
        );
        let SurfaceSessionCreateOutcome::Renderable(surface) =
            device.create_surface_session(&descriptor).unwrap()
        else {
            panic!("expected renderable surface");
        };
        let SurfaceAcquireOutcome::Acquired(frame) =
            device.acquire_surface_frame(surface.session()).unwrap()
        else {
            panic!("expected acquired frame");
        };
        let mut commands = device
            .create_command_list(RenderQueueClass::Graphics, "graph-surface")
            .unwrap();
        commands.begin_render_pass(
            "clear",
            vec![RenderPassColorAttachmentDesc::new(
                frame.target(),
                RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
                RenderPassStoreOp::Store,
            )],
            None,
        );
        commands.end_render_pass();
        let (receipt, weak, releases) = receipt(&device, RenderQueueClass::Graphics);
        let packet = device
            .create_submission_packet(RenderQueueClass::Graphics, vec![commands])
            .unwrap();
        let ticket = device
            .enqueue_submission_packet(packet.with_graph_execution_receipt(receipt).unwrap())
            .unwrap();
        assert_live(&weak, &releases);
        match terminal {
            0 => device.discard_surface_frame(frame).unwrap(),
            1 => device.destroy_surface_session(surface.session()).unwrap(),
            _ => {
                device
                    .reconfigure_surface_session(surface.session(), &descriptor.swapchain)
                    .unwrap();
            }
        }
        assert_eq!(
            device.submission_status(ticket).unwrap(),
            SubmissionStatus::Cancelled
        );
        assert_released(&weak, &releases);
        assert_eq!(device.flush_submissions().unwrap(), 0);
        device.poll_submissions().unwrap();
        assert_eq!(
            device.submission_status(ticket).unwrap(),
            SubmissionStatus::Cancelled
        );
        assert_released(&weak, &releases);
    }
}
