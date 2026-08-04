use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_image_bindings_are_instance_cached() {
    let image = read_runtime_src("graphics/scene/scene_renderer/ui/image.rs");
    let geometry = read_runtime_src("graphics/scene/scene_renderer/ui/render/geometry.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/image/tests.rs");

    assert_contains_all(
        "screen-space UI image binding cache",
        &image,
        &[
            "image_bindings: ScreenSpaceUiImageBindingCache",
            "prepared_textures: ScreenSpaceUiImagePrepareTextureCache",
            "struct ScreenSpaceUiImageBindingCache",
            "struct ScreenSpaceUiImagePrepareTextureCache",
            "bindings: HashMap<usize, CachedScreenSpaceUiImageBinding>",
            "textures: HashMap<ResourceId, Arc<GpuTextureResource>>",
            "texture: Arc<GpuTextureResource>",
            "bind_group: Arc<wgpu::BindGroup>",
            "let key = Arc::as_ptr(texture) as usize;",
            "Arc::ptr_eq(&cached.texture, texture)",
            "cached.last_prepare_epoch = prepare_epoch;",
            "return Arc::clone(&cached.bind_group);",
            "fn begin_prepare(&mut self) -> u64",
            "image_bindings.begin_prepare()",
            "fn clear_frame_state(&mut self)",
            "fn clear(&mut self)",
            "self.bindings = HashMap::new();",
            "fn reset(&mut self)",
            "self.textures = HashMap::new();",
            "fn clear_cpu_staging(&mut self)",
            "self.vertices = Vec::new();",
            "prepared_textures.clear();",
            "fn texture_for(",
            "or_insert_with(|| streamer.ui_texture(requested))",
            "let scissor = image_batch_scissor(batch.frame, viewport, batch.clip_frame)?;",
            "fn image_batch_scissor(",
            "super::render::clipped_scissor(",
            "super::render::frame_to_scissor(viewport)?",
            "fn retain_prepare_epoch(&mut self, prepare_epoch: u64)",
            "image_bindings.retain_prepare_epoch(prepare_epoch);",
            "self.image_bindings.clear();",
            "self.prepared_textures.clear();",
            "pass.set_bind_group(0, image.bind_group.as_ref(), &[]);",
        ],
    );
    assert_eq!(
        image.matches("device.create_bind_group(").count(),
        1,
        "screen-space UI image bindings must have one cache-owned creation site"
    );
    assert!(
        !image.contains("&& let"),
        "screen-space UI image bindings must remain compatible with the Rust 2021 workspace edition"
    );
    let clear_frame_state = image
        .split("pub(super) fn clear_frame_state(&mut self) {")
        .nth(1)
        .and_then(|tail| tail.split("pub(super) fn prepare(").next())
        .expect("screen-space UI image system should own an idle frame-state reset");
    assert_contains_all(
        "screen-space UI image idle reset",
        clear_frame_state,
        &[
            "self.image_bindings.clear();",
            "self.prepared_textures.reset();",
            "self.image_vertices.clear_cpu_staging();",
        ],
    );
    assert!(
        !clear_frame_state.contains("self.image_vertices.buffer = None"),
        "an empty UI frame should retain the reusable image vertex buffer"
    );
    assert!(
        !image.contains(".and_then(|clip| viewport.intersection(clip))\n            .unwrap_or(viewport)"),
        "an image clip outside the viewport must skip the batch instead of drawing with the full viewport scissor"
    );
    assert_contains_all(
        "screen-space UI shared clip geometry",
        &geometry,
        &[
            "pub(in crate::graphics::scene::scene_renderer::ui) fn clipped_scissor(",
            "let visible_frame = viewport.intersection(frame)?;",
            "Some(clip) => visible_frame.intersection(clip).and_then(frame_to_scissor),",
        ],
    );
    assert_contains_all(
        "screen-space UI image clip regression tests",
        &tests,
        &[
            "fn screen_space_ui_image_clip_outside_viewport_skips_the_batch()",
            "fn screen_space_ui_image_without_clip_uses_the_full_viewport_scissor()",
            "fn screen_space_ui_image_clip_intersects_the_viewport_before_scissoring()",
            "fn screen_space_ui_image_clip_missing_the_image_frame_skips_the_batch()",
            "fn screen_space_ui_image_binding_cache_idle_clear_releases_bucket_storage()",
            "fn screen_space_ui_image_idle_clear_releases_transient_cpu_staging()",
        ],
    );

    let line_count = image.lines().count();
    assert!(
        line_count < 800,
        "scene_renderer/ui/image.rs should stay under the R1.4 owner budget, got {line_count}"
    );
}
