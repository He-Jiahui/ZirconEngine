use crate::{wgpu_backend_caps, DeterministicRhiContractDevice};
use zr_rhi::{RenderDevice, RenderQueueClass};

#[test]
fn deterministic_rhi_contract_device_reports_only_implemented_test_capabilities() {
    let device = DeterministicRhiContractDevice::new_headless();

    let status = device.debug_instrumentation_status();
    let caps = device.caps();

    assert_eq!(status.backend_name, "deterministic-rhi-contract-test");
    assert!(status.debug_markers_supported);
    assert!(status.debug_groups_supported);
    assert!(!status.graphics_debugger_capture_supported);
    assert!(!status.active_graphics_debugger_capture);
    assert_eq!(status.last_error, None);
    assert!(!caps.supports_surface);
    assert!(!caps.supports_async_compute);
    assert!(!caps.supports_async_copy);
    assert!(!caps.supports_indirect_draw);
    assert!(!caps.supports_graphics_debugger_capture);
}

#[test]
fn wgpu_capability_mapping_keeps_debug_hooks_independent_from_surface_support() {
    let headless_caps = wgpu_backend_caps(
        "wgpu-headless",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        false,
        true,
    );
    let surface_caps = wgpu_backend_caps(
        "wgpu-surface",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        true,
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
