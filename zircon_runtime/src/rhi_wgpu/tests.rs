use crate::rhi::RenderQueueClass;

use super::wgpu_backend_caps;

#[test]
fn wgpu_caps_fall_back_to_graphics_and_copy_without_rt() {
    let caps = wgpu_backend_caps("wgpu-test", wgpu::Features::empty(), true);

    assert!(caps.supports_queue(RenderQueueClass::Graphics));
    assert!(caps.supports_queue(RenderQueueClass::Copy));
    assert!(!caps.acceleration_structures.supported);
}

#[test]
fn native_ui_surface_source_uses_direct_surface_without_offscreen_blit() {
    let ui_surface = include_str!("ui_surface.rs");
    let pipeline = include_str!("ui_surface/pipeline.rs");
    let retained_cache = include_str!("ui_surface/retained_cache.rs");
    let combined = format!("{ui_surface}\n{pipeline}\n{retained_cache}");

    for forbidden in [
        concat!("zircon-ui-", "offscreen"),
        concat!("blit_", "offscreen_to_surface"),
        concat!("Wgpu", "OffscreenTarget"),
        concat!("Wgpu", "BlitResources"),
    ] {
        assert!(
            !combined.contains(forbidden),
            "native UI surface source must not contain `{forbidden}`"
        );
    }

    assert!(
        combined.contains("WgpuRetainedSurfaceCache"),
        "native UI surface damage must use the retained cache restore path"
    );
}
