use zr_rhi::RenderQueueClass;

use crate::wgpu_backend_caps;

#[test]
fn wgpu_caps_fall_back_to_graphics_and_copy_without_rt() {
    let caps = wgpu_backend_caps(
        "wgpu-test",
        wgpu::Features::empty(),
        wgpu::Limits::default(),
        true,
        true,
    );

    assert!(caps.supports_queue(RenderQueueClass::Graphics));
    assert!(caps.supports_queue(RenderQueueClass::Compute));
    assert!(caps.supports_queue(RenderQueueClass::Copy));
    assert_eq!(
        caps.max_storage_buffers_per_shader_stage,
        wgpu::Limits::default().max_storage_buffers_per_shader_stage
    );
    assert_eq!(
        caps.max_storage_buffer_binding_size,
        u64::from(wgpu::Limits::default().max_storage_buffer_binding_size)
    );
    assert!(caps.supports_fragment_writable_storage);
    assert!(!caps.acceleration_structures.supported);
    assert!(!caps.supports_neural_compute);
    assert!(!caps.supports_sparse_texture);
    assert!(caps.supports_debug_markers);
    assert!(caps.supports_debug_groups);
    assert!(caps.supports_graphics_debugger_capture);
}

#[test]
fn native_ui_surface_source_uses_direct_surface_without_offscreen_blit() {
    let ui_surface = include_str!("../ui_surface.rs");
    let pipeline = include_str!("../ui_surface/pipeline.rs");
    let retained_cache = include_str!("../ui_surface/retained_cache.rs");
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

#[test]
fn command_copy_execution_does_not_clone_whole_source_resources() {
    let source = include_str!("../command_validation.rs");
    let compact = source.split_whitespace().collect::<String>();

    assert!(
        !compact.contains("contents[source_start..source_end].to_vec();"),
        "buffer-to-buffer execution must not allocate a temporary byte vector"
    );
    assert!(
        !compact.contains(".contents.clone();"),
        "buffer-to-texture execution must borrow source contents instead of cloning the whole buffer"
    );
    assert!(
        !compact.contains("letsource_texture=state.textures.get(source).ok_or(RhiError::UnknownTexture(source.raw()))?.clone();"),
        "texture-to-buffer execution must borrow the source texture instead of cloning the whole resource"
    );
}
