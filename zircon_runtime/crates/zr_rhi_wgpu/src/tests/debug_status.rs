use crate::{wgpu_backend_caps, DeterministicRhiContractDevice};
use zr_rhi::{
    RenderBackendCaps, RenderDevice, RenderOperation, RenderOperationSupport, RenderQueueClass,
    RhiError,
};

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
    assert!(caps.supports_surface);
    assert!(!caps.supports_async_compute);
    assert!(!caps.supports_async_copy);
    assert!(caps.supports_indirect_draw);
    assert!(!caps.supports_graphics_debugger_capture);
}

#[test]
fn deterministic_rhi_contract_device_advertises_its_executable_neutral_operations() {
    let device = DeterministicRhiContractDevice::new_headless();

    for operation in [
        RenderOperation::DirectDraw,
        RenderOperation::IndexedDraw,
        RenderOperation::IndirectDraw,
        RenderOperation::MultiDrawIndirect,
        RenderOperation::MultiDrawIndirectCount,
        RenderOperation::ComputeDispatch,
        RenderOperation::ComputeDispatchIndirect,
        RenderOperation::BufferToBufferCopy,
        RenderOperation::BufferToTextureCopy,
        RenderOperation::TextureToBufferCopy,
        RenderOperation::TextureToTextureCopy,
        RenderOperation::DebugMarker,
        RenderOperation::DebugGroup,
    ] {
        assert_eq!(
            device.require_operation(operation),
            Ok(RenderOperationSupport::Native),
            "test device must advertise the neutral command it implements: {operation:?}"
        );
    }

    for operation in [
        RenderOperation::AsyncComputeQueue,
        RenderOperation::AsyncCopyQueue,
        RenderOperation::GraphicsDebuggerCapture,
    ] {
        assert_eq!(
            device.caps().operation_support(operation),
            RenderOperationSupport::Unsupported,
            "test device must not advertise a neutral command without an implementation: {operation:?}"
        );
    }
}

#[test]
fn deterministic_rhi_contract_submit_rejects_a_recorded_operation_missing_from_caps() {
    let device = DeterministicRhiContractDevice::new_headless_with_caps(
        RenderBackendCaps::new("unsupported-debug-marker").with_queue(RenderQueueClass::Graphics),
    );
    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "unsupported-debug-marker")
        .expect("the graphics queue is available for this admission test");
    command_list.push_debug_marker("must be rejected before backend validation");

    assert_eq!(
        device.submit(command_list),
        Err(RhiError::UnsupportedOperation {
            operation: RenderOperation::DebugMarker,
            support: RenderOperationSupport::Unsupported,
        })
    );
}

#[test]
fn wgpu_capability_mapping_keeps_debug_hooks_independent_from_surface_support() {
    let headless_caps = wgpu_backend_caps(
        "wgpu-headless",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        false,
        true,
        true,
    );
    let surface_caps = wgpu_backend_caps(
        "wgpu-surface",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        true,
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
