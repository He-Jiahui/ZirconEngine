use super::*;

#[test]
fn every_native_context_has_one_render_device_and_typed_completion_owner() {
    let source = include_str!("../../ui_surface.rs");

    assert!(source.contains("enum WgpuUiSurfaceCompletionOwner"));
    assert!(source.contains("completion_owner: WgpuUiSurfaceCompletionOwner"));
    assert!(source.contains("WgpuUiSurfaceCompletionOwner::External"));
    assert!(source.contains("WgpuUiSurfaceCompletionOwner::Local"));
    assert!(source.contains("render_device: Arc<WgpuRenderDevice>"));
    assert!(!source.contains("shared_render_device: Option<Arc<WgpuRenderDevice>>"));
    assert!(!source.contains("owns_completion_timeline: bool"));
    assert!(source.contains("gpu_readback_queue: Option<GpuReadbackQueue>"));
    assert!(source.contains("descriptor.allow_gpu_timing && context.completion_owner.is_local()"));
    assert!(source.contains("request_owned_render_device("));
    assert!(source.contains(".with_local_completion_owner()"));

    let presentation = include_str!("../presentation.rs");
    assert!(presentation.contains("self.poll_local_completion_timeline()?"));
    assert!(presentation.contains("readback_queue.collect_completed_after_device_poll()"));
    assert!(!presentation.contains("readback_queue.poll_completed()"));

    let present = presentation
        .split("pub(super) fn present(")
        .nth(1)
        .and_then(|source| source.split("fn resize_if_needed(").next())
        .expect("native present owner");
    let poll = present
        .find("self.poll_local_completion_timeline()?")
        .expect("local completion must be advanced at frame entry");
    let acquire = present
        .find("self.acquire_surface_texture()?")
        .expect("surface acquire must remain explicit");
    assert!(poll < acquire);

    let readback = include_str!("../../gpu_readback_queue/queue.rs");
    assert!(readback.contains("pub(crate) fn collect_completed_after_device_poll("));
    assert!(readback.contains("#[cfg(test)]\n    pub fn poll_completed("));
}

#[test]
fn standalone_and_offscreen_devices_share_the_initial_profile_factory() {
    let surface_setup = include_str!("../surface_setup.rs");
    let offscreen = include_str!(
        "../../../../../src/graphics/backend/render_backend/render_backend_new_offscreen.rs"
    );
    let profile = include_str!("../../device_profile.rs");

    assert!(profile.contains("pub fn initial_wgpu_render_device_profile("));
    assert!(surface_setup.contains("initial_wgpu_render_device_profile("));
    assert!(offscreen.contains("initial_wgpu_render_device_profile("));
    assert!(!surface_setup.contains("RenderDeviceProfile::new("));
    assert!(!offscreen.contains("RenderDeviceProfile::new("));
}

#[test]
fn image_pins_retire_with_their_submission_ticket() {
    let source = include_str!("../../ui_surface.rs");
    let presentation = include_str!("../presentation.rs");
    let packet = include_str!("../../production/device/native_recording.rs");
    let submission = include_str!("../../production/submission.rs");
    let retirement = include_str!("../../production/submission/ui_image_retirement.rs");

    assert!(!source.contains("on_submitted_work_done"));
    assert!(source.contains("packet.retain_ui_image_pins(pins)"));
    assert!(presentation.contains("image_allocation_pins"));
    assert!(presentation
        .contains("self.submit_present_command_buffer(encoder.finish(), image_allocation_pins)"));
    assert!(packet.contains("ui_image_pins: Option<WgpuUiImageInFlightPins>"));
    assert!(packet.contains("pub(crate) fn retain_ui_image_pins("));
    assert!(submission.contains("WgpuUiImageRetirementOwner"));
    assert!(submission.contains("commit_packet_with_ui_image_pins("));
    assert!(submission
        .split_whitespace()
        .collect::<String>()
        .contains("self.ui_image_retirements.retain_batch("));
    assert!(submission.contains("completion_retirements.complete(&completed_tickets)"));
    assert!(submission.contains("self.ui_image_retirements.terminalize_all()"));
    assert!(retirement.contains("HashMap<SubmissionTicket, WgpuUiImageInFlightPins>"));
}

#[test]
fn shared_surface_present_uses_the_arc_device_owner_and_keeps_its_ticket() {
    let backend =
        include_str!("../../../../../src/graphics/backend/render_backend/render_backend.rs");
    let device = include_str!("../../production/device.rs");
    let source = include_str!("../../ui_surface.rs");
    let presentation = include_str!("../presentation.rs");
    let neutral_stats = include_str!("../../../../zr_rhi/src/ui_surface.rs");

    assert!(backend.contains("render_device: Arc<WgpuRenderDevice>"));
    assert!(device.contains("pub fn ui_surface_context(self: &Arc<Self>)"));
    assert!(source.contains("render_device: Arc<WgpuRenderDevice>"));
    assert!(source.contains("fn submit_present_command_buffer("));
    assert!(source.contains(".submit_native_recording_packet(packet)"));
    assert!(!source.contains("self.queue.submit(Some(command_buffer))"));
    assert!(!source.contains("Result<Option<SubmissionTicket>, RhiError>"));
    assert!(presentation
        .split_whitespace()
        .collect::<String>()
        .contains("letsubmission=Some(self.submit_present_command_buffer"));
    assert!(!presentation.contains("self.queue.submit(Some(encoder.finish()));"));
    assert!(neutral_stats.contains("pub submission: Option<SubmissionTicket>"));
    assert!(source.contains("stats.submission = presentation.submission"));

    let submit = presentation
        .find("self.submit_present_command_buffer(")
        .expect("shared UI present must submit through the device owner");
    let retained_commit = presentation
        .find("retained_cache.mark_ordinary_baseline_ready()")
        .expect("retained cache commit must remain explicit");
    let native_present = presentation
        .find("surface_texture.present()")
        .expect("native present must remain explicit");
    assert!(submit < retained_commit);
    assert!(submit < native_present);
}
