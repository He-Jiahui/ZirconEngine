#[test]
fn surface_bootstrap_transfers_one_compatible_native_surface_into_the_neutral_device() {
    let bootstrap = include_str!("../surface_bootstrap.rs");
    let selected_adapter = include_str!("../surface_bootstrap_adapter.rs");
    let device = include_str!("../device.rs");
    let service = include_str!("../surface.rs");

    for required in [
        "pub struct WgpuSurfaceBootstrap",
        "instance: wgpu::Instance",
        "surface: wgpu::Surface<'static>",
        "pub fn select_compatible_adapter(\n        self,",
        "surface.get_capabilities(&adapter)",
        "surface_descriptor_is_supported(&capabilities, &descriptor)",
    ] {
        assert!(
            bootstrap.contains(required),
            "surface bootstrap must retain the one-way handoff step `{required}`"
        );
    }

    for required in [
        "pub struct WgpuSurfaceAdapterBootstrap",
        "pub fn adapter(&self) -> &wgpu::Adapter",
        "pub fn request_render_device(",
        "pollster::block_on(adapter.request_device(native_descriptor))",
        "WgpuRenderDeviceContext::new(instance, adapter, device, queue)",
        "WgpuRenderDevice::new(context, profile)?",
        "render_device.adopt_surface_session(descriptor, surface)?",
    ] {
        assert!(
            selected_adapter.contains(required),
            "selected-adapter bootstrap must retain the one-way handoff step `{required}`"
        );
    }

    assert!(
        device.contains("fn adopt_surface_session("),
        "the neutral device must own the bootstrap handoff endpoint"
    );
    assert!(
        service.contains("fn adopt_session("),
        "the surface service must register an adopted surface instead of recreating it"
    );
    assert!(
        service.contains("fn surface_descriptor_is_supported("),
        "surface bootstrap selection must use the same format and present-mode contract as session creation"
    );
    let frames = service
        .find("frames: HashMap<SurfaceFrameId, WgpuSurfaceFrame>")
        .expect("surface service must track acquired frame ownership");
    let sessions = service
        .find("sessions: HashMap<SurfaceSession, WgpuSurfaceSession>")
        .expect("surface service must track native surface ownership");
    assert!(
        frames < sessions,
        "acquired SurfaceTexture frames must drop before their owning native surfaces"
    );
    assert!(
        !bootstrap.contains("pub fn native_surface"),
        "surface bootstrap must not expose a native surface escape hatch"
    );
    assert!(
        !bootstrap.contains("pub fn into_surface"),
        "surface bootstrap must transfer native ownership only into WgpuRenderDevice"
    );
    assert!(
        !bootstrap.contains("pub fn request_render_device"),
        "only the selected surface-compatible adapter may create and transfer a render device"
    );
    assert!(
        !selected_adapter.contains("pub fn native_surface"),
        "selected-adapter bootstrap must not expose a native surface escape hatch"
    );
    assert!(
        !selected_adapter.contains("pub fn into_surface"),
        "selected-adapter bootstrap must retain native surface ownership until handoff"
    );
    assert!(
        !selected_adapter.contains("device: wgpu::Device"),
        "selected-adapter bootstrap must request its own native device rather than accept one"
    );
    assert!(
        !selected_adapter.contains("queue: wgpu::Queue"),
        "selected-adapter bootstrap must request its own native queue rather than accept one"
    );
    let profile_guard = selected_adapter
        .find("if profile.adapter() != &adapter_facts")
        .expect("selected-adapter bootstrap must reject a profile for another adapter");
    let device_request = selected_adapter
        .find("adapter.request_device(native_descriptor)")
        .expect("selected-adapter bootstrap must request a device from its stored adapter");
    assert!(
        profile_guard < device_request,
        "profile adapter validation must fail before requesting a native device"
    );
}
