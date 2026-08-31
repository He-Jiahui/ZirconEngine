use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    CommandList, PresentMode, RenderClearColor, RenderDevice, RenderNativeSurfaceTarget,
    RenderPassColorAttachmentDesc, RenderPassColorLoadOp, RenderPassStoreOp,
    RenderPassTextureViewDesc, RenderQueueClass, RenderSurfaceDescriptor, SurfaceAcquireOutcome,
    SurfaceSessionCreateOutcome, SwapchainDesc, TextureFormat,
};

fn surface_descriptor(width: u32, height: u32) -> RenderSurfaceDescriptor {
    RenderSurfaceDescriptor::new(
        "deterministic-surface",
        RenderNativeSurfaceTarget::Win32 {
            hwnd: 1,
            hinstance: None,
        },
        SwapchainDesc {
            width,
            height,
            present_mode: PresentMode::Fifo,
            format: TextureFormat::Bgra8UnormSrgb,
        },
    )
}

#[test]
fn surface_frame_lease_requires_submission_and_releases_only_through_present() {
    let device = DeterministicRhiContractDevice::new_headless();
    let SurfaceSessionCreateOutcome::Renderable(surface) = device
        .create_surface_session(&surface_descriptor(64, 64))
        .expect("deterministic device should create a renderable surface")
    else {
        panic!("nonzero deterministic surface must be renderable");
    };
    let SurfaceAcquireOutcome::Acquired(frame) = device
        .acquire_surface_frame(surface.session())
        .expect("deterministic surface should acquire one frame")
    else {
        panic!("deterministic surface should not need retry or reconfiguration");
    };

    assert!(device.destroy_texture(frame.target()).is_err());
    assert!(device.destroy_texture_view(frame.default_view()).is_err());

    let mut unrelated = device
        .create_command_list(RenderQueueClass::Graphics, "surface-unrelated")
        .unwrap();
    unrelated.push_debug_marker("unrelated-submission");
    let unrelated_ticket = device.submit(unrelated).unwrap();
    assert!(matches!(
        device.present_surface_frame(frame.clone(), unrelated_ticket),
        Err(zr_rhi::RhiError::SurfaceFrameSubmissionMissingTarget { .. })
    ));

    let mut commands = device
        .create_command_list(RenderQueueClass::Graphics, "surface-clear")
        .unwrap();
    commands.begin_render_pass(
        "surface-clear-pass",
        vec![RenderPassColorAttachmentDesc::new(
            frame.target(),
            RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
            RenderPassStoreOp::Store,
        )
        .with_view(
            RenderPassTextureViewDesc::new(frame.target())
                .with_registered_view(frame.default_view()),
        )],
        None,
    );
    commands.end_render_pass();
    let ticket = device.submit(commands).unwrap();

    let target = frame.target();
    let default_view = frame.default_view();
    device
        .present_surface_frame(frame, ticket)
        .expect("submitted surface frame should present exactly once");
    assert!(device.texture_desc(target).is_err());
    assert!(device.texture_view_desc(default_view).is_err());
}

#[test]
fn surface_reconfigure_invalidates_old_session_and_zero_extent_is_non_renderable() {
    let device = DeterministicRhiContractDevice::new_headless();
    let SurfaceSessionCreateOutcome::Renderable(surface) = device
        .create_surface_session(&surface_descriptor(32, 32))
        .unwrap()
    else {
        panic!("nonzero deterministic surface must be renderable");
    };
    let SurfaceAcquireOutcome::Acquired(old_frame) =
        device.acquire_surface_frame(surface.session()).unwrap()
    else {
        panic!("renderable deterministic surface must acquire before reconfigure");
    };
    let mut pending_commands = device
        .create_command_list(RenderQueueClass::Graphics, "surface-reconfigure-pending")
        .unwrap();
    pending_commands.begin_render_pass(
        "surface-reconfigure-pending-pass",
        vec![RenderPassColorAttachmentDesc::new(
            old_frame.target(),
            RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
            RenderPassStoreOp::Store,
        )],
        None,
    );
    pending_commands.end_render_pass();
    let pending_ticket = device.enqueue_command_list(pending_commands).unwrap();
    let SurfaceSessionCreateOutcome::Renderable(reconfigured) = device
        .reconfigure_surface_session(surface.session(), &surface_descriptor(128, 72).swapchain)
        .unwrap()
    else {
        panic!("nonzero reconfigured deterministic surface must be renderable");
    };

    assert!(device.acquire_surface_frame(surface.session()).is_err());
    assert_ne!(surface.session(), reconfigured.session());
    assert_eq!(
        device.submission_status(pending_ticket).unwrap(),
        zr_rhi::SubmissionStatus::Cancelled
    );
    assert_eq!(device.flush_submissions().unwrap(), 0);
    assert!(device.texture_desc(old_frame.target()).is_err());
    assert!(device.discard_surface_frame(old_frame).is_err());

    let SurfaceSessionCreateOutcome::NonRenderable(non_renderable) = device
        .create_surface_session(&surface_descriptor(0, 0))
        .unwrap()
    else {
        panic!("zero extent must remain non-renderable");
    };
    assert!(matches!(
        device
            .acquire_surface_frame(non_renderable.session())
            .unwrap(),
        SurfaceAcquireOutcome::NonRenderable { .. }
    ));
}
