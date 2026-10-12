#[test]
fn neutral_mvp_renderer_transfers_native_context_without_a_raw_backend_owner() {
    let source = include_str!("../neutral_mvp_renderer.rs");

    assert!(source.contains("WgpuRenderDeviceContext::new("));
    assert!(source.contains("WgpuRenderDevice::new(context, profile)"));
    assert!(source.contains("WgpuMvpOffscreenTriangle::new(&device"));
    assert!(source.contains("self.frame.submit(&self.device)"));
    assert!(!source.contains("RenderBackend::new_offscreen"));
    assert!(!source.contains("queue.submit"));
    assert!(!source.contains("WgpuDeviceErrorSupervisor::install"));
}
