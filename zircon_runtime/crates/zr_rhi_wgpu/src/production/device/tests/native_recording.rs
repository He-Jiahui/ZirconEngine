use std::sync::Arc;

use zr_rhi::{
    DeviceGeneration, DeviceId, RenderQueueClass, RhiError, RhiGraphAccessId, RhiGraphAccessRange,
    RhiGraphExecutionAccess, RhiGraphExecutionPass, RhiGraphExecutionReceipt,
    RhiGraphPhysicalResourceLease, RhiGraphQueueLane, RhiGraphResourceAccessKind,
    RhiGraphResourceBounds, RhiGraphResourceId, RhiGraphResourceKind, RhiGraphResourceState,
};

use super::WgpuNativeSubmissionPacket;

fn queue_mismatch_receipt() -> RhiGraphExecutionReceipt {
    let device_id = DeviceId::new(41);
    let generation = DeviceGeneration::new(9);
    let access_id = RhiGraphAccessId::new(0, 7, 0);
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::Texture, 0, 7);
    let access = RhiGraphExecutionAccess::new(
        access_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Write,
        RhiGraphAccessRange::texture(0, 1, 0, 1, 1),
        RhiGraphResourceState::ColorAttachment,
        RhiGraphQueueLane::Graphics,
        Some(42),
        true,
    );
    let pass = RhiGraphExecutionPass::new(
        0,
        0,
        7,
        RhiGraphQueueLane::Graphics,
        vec![access],
        Vec::new(),
    );
    let lease = RhiGraphPhysicalResourceLease::new(
        access_id,
        resource,
        Some(42),
        device_id,
        generation,
        RhiGraphResourceBounds::texture(1, 1, 1).expect("valid texture bounds"),
        Arc::new(()),
    );
    RhiGraphExecutionReceipt::new(
        device_id,
        generation,
        11,
        7,
        RenderQueueClass::Compute,
        vec![pass],
        vec![lease],
    )
    .expect("typed queue mismatch receipt")
}

#[test]
fn graph_receipt_queue_mismatch_is_reported_as_typed_queue_error() {
    let packet = WgpuNativeSubmissionPacket {
        device_id: DeviceId::new(41),
        generation: DeviceGeneration::new(9),
        queue_class: RenderQueueClass::Graphics,
        command_buffers: Vec::new(),
        ui_image_pins: None,
        graph_execution_receipt: None,
    };

    let error = match packet.with_graph_execution_receipt(queue_mismatch_receipt()) {
        Ok(_) => panic!("queue-only mismatch must be rejected"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        RhiError::SubmissionPacketQueueMismatch {
            packet_queue: RenderQueueClass::Graphics,
            command_queue: RenderQueueClass::Compute,
        }
    ));
}

#[test]
fn native_recorder_never_exposes_queue_poll_or_flush_authority() {
    let source = include_str!("../native_recording.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production native recorder source");

    assert!(!production.contains("wgpu::Queue"));
    assert!(!production.contains(".poll("));
    assert!(!production.contains(".flush("));
    assert!(!production.contains("queue.submit"));
    assert!(production.contains("self.submissions.begin_packet(queue_class)"));
    assert!(production.contains("self.submissions.commit_packet_with_ui_image_pins("));
}

#[test]
fn native_packet_is_generation_qualified_and_opaque_to_product_callers() {
    let source = include_str!("../native_recording.rs");
    let source = source.split("mod tests {").next().unwrap();

    for field in [
        "device_id",
        "generation",
        "queue_class",
        "command_buffers",
        "ui_image_pins",
    ] {
        assert!(source.contains(&format!("    {field}:")));
    }
    assert!(source.contains("fn into_submission_parts("));
    assert!(!source.contains("pub fn into_submission_parts("));
    assert!(source.contains("RhiError::SubmissionPacketDeviceMismatch"));
    assert!(source.contains("RhiError::EmptySubmissionPacket"));
}

#[test]
fn fused_surface_target_is_registered_before_the_scene_packet_flushes() {
    let source = include_str!("../native_recording.rs");
    let source = source.split("mod tests {").next().unwrap();
    let fused_submit = source
        .split("pub fn submit_native_recording_packet_with_frame_diagnostics_and_surface")
        .nth(1)
        .expect("fused surface submission owner");
    let register = fused_submit
        .find("self.register_native_surface_frame_use(surface_target.frame_lease(), ticket)")
        .expect("surface lease must retain the scene ticket");
    let validate_owner = fused_submit
        .find("surface_target.validate_owner(self)")
        .expect("surface target must belong to the submitting device owner");
    let enqueue = fused_submit
        .find("self.enqueue_native_recording_packet_with_frame_diagnostics(")
        .expect("scene packet enqueue");
    let flush = fused_submit
        .find("self.flush_submissions()")
        .expect("fused scene packet must flush once");

    assert!(validate_owner < enqueue);
    assert!(enqueue < register);
    assert!(register < flush);
    assert!(fused_submit[..flush].contains("self.cancel_accepted_packet(ticket)"));
    assert!(!fused_submit.contains("queue.submit"));
}
