use super::{clamp_surface_size, wgpu_surface_format, ViewportSurfacePresentOutcome};
use crate::core::math::UVec2;
use crate::rhi::{DeviceGeneration, DeviceId, RenderQueueClass, SubmissionTicket, TextureFormat};

#[test]
fn viewport_surface_modes_map_without_losing_requested_policy() {
    use crate::core::framework::render::RenderViewportSurfaceDescriptor;
    use crate::core::framework::window::WindowPresentMode as W;
    use crate::rhi::{PresentMode as R, RenderNativeSurfaceTarget};
    let descriptor = RenderViewportSurfaceDescriptor::new(
        UVec2::new(1, 1),
        RenderNativeSurfaceTarget::Win32 {
            hwnd: 73,
            hinstance: None,
        },
    );
    assert_eq!(descriptor.present_mode, W::Fifo);
    for (requested, backend) in [
        (W::Fifo, R::Fifo),
        (W::Immediate, R::Immediate),
        (W::Mailbox, R::Mailbox),
        (W::FifoRelaxed, R::FifoRelaxed),
        (W::AutoVsync, R::AutoVsync),
        (W::AutoNoVsync, R::AutoNoVsync),
    ] {
        let changed = descriptor.with_present_mode(requested);
        assert_eq!(super::requested_present_mode(changed.present_mode), backend);
        assert_eq!(
            (changed.target, changed.size),
            (descriptor.target, descriptor.size)
        );
    }
    let source = include_str!("../viewport_surface.rs");
    assert!(source.contains("requested_present_mode: surface_descriptor.swapchain.present_mode"));
    assert!(source.contains("present_mode: self.requested_present_mode"));
    assert!(source.contains(".reconfigure_surface_session(self.session.session(), &requested)"));
}

#[test]
fn graphics_surface_backend_clamps_zero_descriptor_size() {
    assert_eq!(clamp_surface_size(UVec2::new(0, 0)), UVec2::new(1, 1));
    assert_eq!(clamp_surface_size(UVec2::new(640, 0)), UVec2::new(640, 1));
    assert_eq!(clamp_surface_size(UVec2::new(0, 480)), UVec2::new(1, 480));
}

#[test]
fn graphics_surface_backend_accepts_only_neutral_srgb_formats() {
    assert_eq!(
        wgpu_surface_format(TextureFormat::Bgra8UnormSrgb).unwrap(),
        wgpu::TextureFormat::Bgra8UnormSrgb
    );
    assert_eq!(
        wgpu_surface_format(TextureFormat::Rgba8UnormSrgb).unwrap(),
        wgpu::TextureFormat::Rgba8UnormSrgb
    );
    assert!(wgpu_surface_format(TextureFormat::Rgba16Float).is_err());
}

#[test]
fn graphics_surface_backend_uses_the_neutral_surface_transaction_owner() {
    let source = include_str!("../viewport_surface.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("surface source must retain its test-module boundary");

    assert!(production.contains(".create_surface_session(&surface_descriptor)"));
    assert!(
        production.contains("SurfaceBlitResources::new(&self.device, session.swapchain.format)")
    );
    assert!(!production.contains("BGRA8 sRGB is part of the neutral surface contract"));
    assert!(!production.contains("RGBA8 sRGB is part of the neutral surface contract"));
    assert!(production.contains(".acquire_surface_frame(self.session.session())"));
    assert!(production.contains("ViewportSurfacePresentOutcome::Presented(submission)"));
    assert!(!production.contains("queue.submit("));
    assert!(!production.contains("get_current_texture"));
    assert!(production.contains("prepare_native_surface_frame_target(frame)"));
    assert!(production.contains("target.record("));
    assert!(production.contains("self.render_device.as_ref(),"));
    assert!(production.contains("target.present(submission)"));
    assert!(production.contains("match target.discard()"));
    assert!(production.contains("fn discard_frame_target("));
    assert!(production.contains("GraphicsError::SurfaceFrameCleanupFailed"));
    assert!(!production.contains("present_texture("));
    assert!(!production.contains("submit_native_surface_recording_packet"));
    assert!(!production.contains("surface.configure"));
    assert!(!production.contains("surface_texture.present"));
}

#[test]
fn only_presented_surface_outcomes_expose_a_submission_ticket() {
    let ticket = SubmissionTicket::new(
        DeviceId::new(3),
        DeviceGeneration::new(2),
        RenderQueueClass::Graphics,
        41,
    );

    assert_eq!(
        ViewportSurfacePresentOutcome::Presented(ticket).submission_ticket(),
        Some(ticket)
    );
    assert_eq!(
        ViewportSurfacePresentOutcome::Reconfigured.submission_ticket(),
        None
    );
    assert_eq!(
        ViewportSurfacePresentOutcome::DeferredTimeout.submission_ticket(),
        None
    );
    assert_eq!(
        ViewportSurfacePresentOutcome::DeferredOccluded.submission_ticket(),
        None
    );
}
