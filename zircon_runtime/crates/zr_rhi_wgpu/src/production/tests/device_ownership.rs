fn assert_declared_drop_order(source: &str, declaration: &str, fields: &[&str]) {
    let body = source
        .split_once(declaration)
        .and_then(|(_, remainder)| remainder.split_once("\n}"))
        .map(|(body, _)| body)
        .unwrap_or_else(|| panic!("missing struct declaration {declaration}"));
    for pair in fields.windows(2) {
        let earlier = body
            .find(pair[0])
            .unwrap_or_else(|| panic!("missing field {} in {declaration}", pair[0]));
        let later = body
            .find(pair[1])
            .unwrap_or_else(|| panic!("missing field {} in {declaration}", pair[1]));
        assert!(
            earlier < later,
            "{} must be declared before {} so Rust drops native dependents before owners",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn production_device_accepts_a_single_opaque_native_context_handoff() {
    let source = include_str!("../device.rs");
    let context_source = include_str!("../device/context.rs");

    for field in [
        "instance: wgpu::Instance",
        "adapter: wgpu::Adapter",
        "device: wgpu::Device",
        "queue: wgpu::Queue",
        "ui_image_registry: Arc<WgpuUiSharedImageRegistry>",
    ] {
        assert!(
            context_source.contains(field),
            "WgpuRenderDeviceContext must retain {field} until its one-shot handoff"
        );
    }
    assert!(source.contains("context: WgpuRenderDeviceContext"));
    assert!(source.contains("pub fn ui_surface_context(self: &Arc<Self>) -> WgpuUiSurfaceContext"));
    let adapter_validation = source
        .find("validate_context_adapter(&adapter, &profile)?")
        .expect("context handoff must validate its adapter profile");
    let fault_install = source
        .find("WgpuDeviceErrorSupervisor::install")
        .expect("neutral device must install its fault supervisor");
    assert!(
        adapter_validation < fault_install,
        "a mismatched profile must fail before installing callbacks on the native device"
    );
    let limits_validation = source
        .find("validate_context_device_limits(&device, &profile)?")
        .expect("context handoff must validate its negotiated device limits");
    assert!(
        limits_validation < fault_install,
        "a mismatched limits receipt must fail before installing callbacks on the native device"
    );
    assert!(
        !source.contains("pub fn native_device"),
        "neutral device ownership must not grow a public raw-device escape hatch"
    );
    assert!(
        !source.contains("pub fn native_queue"),
        "neutral device ownership must not grow a public raw-queue escape hatch"
    );
}

#[test]
fn production_native_dependents_drop_before_queue_device_adapter_and_instance() {
    assert_declared_drop_order(
        include_str!("../device.rs"),
        "pub struct WgpuRenderDevice {",
        &[
            "submissions:",
            "diagnostics:",
            "surfaces:",
            "registry:",
            "ui_image_registry:",
            "queue:",
            "device:",
            "adapter:",
            "instance:",
        ],
    );
    assert_declared_drop_order(
        include_str!("../device/context.rs"),
        "pub struct WgpuRenderDeviceContext {",
        &[
            "ui_image_registry:",
            "queue:",
            "device:",
            "adapter:",
            "instance:",
        ],
    );
    assert_declared_drop_order(
        include_str!("../submission.rs"),
        "pub(crate) struct WgpuSubmissionService {",
        &["state:", "queue_access:", "queue:"],
    );
}
