use zr_rhi::{
    PresentMode, RenderNativeSurfaceTarget, RenderSurfaceDescriptor, SwapchainDesc, TextureFormat,
};

use super::{
    choose_canvas_surface_format, choose_present_mode, surface_descriptor_is_supported,
    surface_present_mode, surface_texture_format,
};

#[test]
fn present_mode_projection_preserves_concrete_fifo_relaxed() {
    assert_eq!(
        surface_present_mode(wgpu::PresentMode::FifoRelaxed),
        PresentMode::FifoRelaxed
    );
}

#[test]
fn auto_present_mode_negotiation_uses_actual_capabilities() {
    use wgpu::PresentMode as W;
    for caps in [
        [W::Fifo, W::Mailbox, W::Immediate],
        [W::Immediate, W::Mailbox, W::Fifo],
    ] {
        assert_eq!(
            choose_present_mode(&caps, PresentMode::AutoNoVsync),
            Some(W::Immediate)
        );
    }
    assert_eq!(
        choose_present_mode(&[W::Fifo, W::Mailbox], PresentMode::AutoNoVsync),
        Some(W::Mailbox)
    );
    assert_eq!(
        choose_present_mode(&[W::Fifo], PresentMode::AutoNoVsync),
        Some(W::Fifo)
    );
    assert_eq!(
        choose_present_mode(&[W::Fifo, W::FifoRelaxed], PresentMode::AutoVsync),
        Some(W::FifoRelaxed)
    );
    assert_eq!(
        choose_present_mode(&[W::Fifo], PresentMode::AutoVsync),
        Some(W::Fifo)
    );
    assert_eq!(
        choose_present_mode(&[W::Mailbox], PresentMode::AutoVsync),
        None
    );
    assert_eq!(choose_present_mode(&[], PresentMode::AutoNoVsync), None);
    assert_eq!(
        choose_present_mode(&[W::AutoNoVsync], PresentMode::AutoNoVsync),
        None
    );
}

#[test]
fn explicit_present_mode_rejects_unavailable_capabilities() {
    use wgpu::PresentMode as W;
    for (request, concrete) in [
        (PresentMode::Immediate, W::Immediate),
        (PresentMode::Fifo, W::Fifo),
        (PresentMode::Mailbox, W::Mailbox),
        (PresentMode::FifoRelaxed, W::FifoRelaxed),
    ] {
        assert_eq!(choose_present_mode(&[concrete], request), Some(concrete));
        assert_eq!(surface_present_mode(concrete), request);
        assert_eq!(choose_present_mode(&[], request), None);
        for other in [W::Immediate, W::Fifo, W::Mailbox, W::FifoRelaxed] {
            if other != concrete {
                assert_eq!(choose_present_mode(&[other], request), None);
            }
        }
    }
}

#[test]
fn native_fifo_variants_project_to_the_neutral_fifo_contract() {
    for mode in [
        wgpu::PresentMode::Fifo,
        wgpu::PresentMode::AutoVsync,
        wgpu::PresentMode::AutoNoVsync,
    ] {
        assert_eq!(surface_present_mode(mode), PresentMode::Fifo);
    }
}

#[test]
fn surface_receipt_format_preserves_sdr_layout_and_encoding() {
    use wgpu::TextureFormat as W;

    assert_eq!(
        [
            W::Bgra8UnormSrgb,
            W::Rgba8UnormSrgb,
            W::Bgra8Unorm,
            W::Rgba8Unorm,
            W::Rgba16Float,
        ]
        .map(surface_texture_format),
        [
            Some(TextureFormat::Bgra8UnormSrgb),
            Some(TextureFormat::Rgba8UnormSrgb),
            Some(TextureFormat::Bgra8Unorm),
            Some(TextureFormat::Rgba8Unorm),
            None,
        ]
    );
}

#[test]
fn canvas_surface_format_preserves_the_first_supported_browser_preference() {
    use wgpu::TextureFormat as W;

    for (reported, chosen, receipt) in [
        (
            &[W::Bgra8Unorm, W::Rgba8Unorm, W::Rgba16Float][..],
            W::Bgra8Unorm,
            TextureFormat::Bgra8Unorm,
        ),
        (
            &[W::Rgba8Unorm, W::Bgra8Unorm, W::Rgba16Float][..],
            W::Rgba8Unorm,
            TextureFormat::Rgba8Unorm,
        ),
        (
            &[W::Rgba16Float, W::Bgra8UnormSrgb, W::Rgba8Unorm][..],
            W::Rgba8Unorm,
            TextureFormat::Rgba8Unorm,
        ),
    ] {
        let selected = choose_canvas_surface_format(reported);
        assert_eq!(selected, Some(chosen), "reported formats: {reported:?}");
        assert_eq!(selected.and_then(surface_texture_format), Some(receipt));
    }
}

#[test]
fn canvas_surface_format_rejects_lists_without_sdr_unorm() {
    use wgpu::TextureFormat as W;

    assert_eq!(choose_canvas_surface_format(&[]), None);
    assert_eq!(
        choose_canvas_surface_format(&[W::Rgba16Float, W::Bgra8UnormSrgb, W::Rgba8UnormSrgb]),
        None
    );
}

#[test]
fn native_surface_admission_still_requires_srgb_and_a_supported_present_mode() {
    use wgpu::TextureFormat as W;

    let mut descriptor = RenderSurfaceDescriptor::new(
        "native-admission-control",
        RenderNativeSurfaceTarget::Win32 {
            hwnd: 1,
            hinstance: None,
        },
        SwapchainDesc {
            width: 320,
            height: 240,
            present_mode: PresentMode::Fifo,
            format: TextureFormat::Bgra8UnormSrgb,
        },
    );
    let mut caps = wgpu::SurfaceCapabilities {
        formats: vec![W::Bgra8UnormSrgb],
        present_modes: vec![wgpu::PresentMode::Fifo],
        alpha_modes: vec![wgpu::CompositeAlphaMode::Opaque],
        usages: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };
    assert!(surface_descriptor_is_supported(&caps, &descriptor));

    caps.formats = vec![W::Rgba8UnormSrgb];
    assert!(surface_descriptor_is_supported(&caps, &descriptor));
    descriptor.swapchain.present_mode = PresentMode::Mailbox;
    assert!(!surface_descriptor_is_supported(&caps, &descriptor));

    descriptor.swapchain.present_mode = PresentMode::Fifo;
    descriptor.swapchain.format = TextureFormat::Bgra8Unorm;
    caps.formats = vec![W::Bgra8Unorm, W::Rgba8Unorm];
    assert!(!surface_descriptor_is_supported(&caps, &descriptor));
}
