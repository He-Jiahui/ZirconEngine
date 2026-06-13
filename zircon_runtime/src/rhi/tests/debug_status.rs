use crate::rhi::{RenderDevice, RenderQueueClass};
use crate::rhi_wgpu::{wgpu_backend_caps, WgpuRenderDevice};

#[test]
fn wgpu_rhi_reports_debug_instrumentation_status_at_device_boundary() {
    let device = WgpuRenderDevice::new_headless();

    let status = device.debug_instrumentation_status();

    assert_eq!(status.backend_name, "wgpu");
    assert!(status.debug_markers_supported);
    assert!(status.debug_groups_supported);
    assert!(status.graphics_debugger_capture_supported);
    assert!(!status.active_graphics_debugger_capture);
    assert_eq!(status.last_error, None);
}

#[test]
fn wgpu_capability_mapping_keeps_debug_hooks_independent_from_surface_support() {
    let headless_caps = wgpu_backend_caps(
        "wgpu-headless",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        false,
    );
    let surface_caps = wgpu_backend_caps(
        "wgpu-surface",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        true,
    );

    assert!(!headless_caps.supports_surface);
    assert!(surface_caps.supports_surface);

    for caps in [&headless_caps, &surface_caps] {
        assert!(caps.supports_queue(RenderQueueClass::Graphics));
        assert!(caps.supports_debug_markers);
        assert!(caps.supports_debug_groups);
        assert!(caps.supports_graphics_debugger_capture);
    }
}
