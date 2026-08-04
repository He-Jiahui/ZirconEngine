use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_native_bitmap_atlas_skips_steady_upload_plan_work() {
    let atlas_frame = read_runtime_src("text/native_bitmap_atlas.rs");
    let renderer = read_runtime_src("graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs");
    let prepare_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/atlas_renderer/prepare_report.rs");
    let renderer_state =
        read_runtime_src("graphics/scene/scene_renderer/ui/atlas_renderer/state.rs");
    let instance_buffer =
        read_runtime_src("graphics/scene/scene_renderer/ui/atlas_renderer/instance_buffer.rs");
    let atlas_tests = read_runtime_src("graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs");
    let text = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let native_layout = read_runtime_src(
        "graphics/scene/scene_renderer/ui/atlas_renderer/product_framebuffer/native_layout.rs",
    );

    assert_contains_all(
        "native bitmap atlas source bytes stay lazy",
        &atlas_frame,
        &[
            "pub(crate) fn source_bytes(&self) -> impl Iterator",
            "GlyphAtlasBitmapUploadSourceBytes::with_face_epoch(",
        ],
    );
    assert_contains_all(
        "native bitmap atlas renderer skips stable upload plans",
        &renderer,
        &[
            "fn prepare_submission_upload<'a, I>(",
            "if submission.upload_commands().is_empty()",
            "GlyphAtlasBitmapTextureUploadFrameReport::default()",
            "GlyphAtlasBitmapPageShadowCommit::default()",
            ".with_upload_plan_preparation(upload_plan_built)",
            "fn with_upload_plan_preparation(mut self, built: bool)",
            "draw_pass.instance_buffer_payload_hash = None;",
            "last_viewport_transform: Option<GlyphAtlasGpuViewportTransform>",
            "glyph_atlas_bitmap_renderer_viewport_uniform_write_required(",
            "self.last_viewport_transform = Some(viewport_transform);",
        ],
    );
    assert_contains_all(
        "native bitmap atlas upload-plan diagnostics",
        &renderer_state,
        &[
            "upload_plan_build_count: usize",
            "upload_plan_skip_count: usize",
        ],
    );
    assert_contains_all(
        "native bitmap atlas instance buffers retain payload identity",
        &renderer_state,
        &["instance_buffer_payload_hash: Option<[u8; 32]>"],
    );
    assert_contains_all(
        "native bitmap atlas instance buffers skip stable GPU writes",
        &instance_buffer,
        &[
            "glyph_atlas_bitmap_renderer_instance_buffer_write_required(",
            "blake3::hash(instance_bytes)",
            "draw_pass.instance_buffer_payload_hash = Some(payload_hash);",
            "queue.write_buffer(instance_buffer, 0, instance_bytes);",
        ],
    );
    assert!(
        !instance_buffer.contains("&& let"),
        "native bitmap atlas instance-buffer code must remain compatible with the Rust 2021 workspace edition"
    );
    assert_contains_all(
        "native bitmap atlas instance buffer stable-frame regression",
        &atlas_tests,
        &[
            "glyph_atlas_bitmap_instance_buffer_writes_only_for_new_payloads_or_reallocation",
            "glyph_atlas_bitmap_viewport_uniform_writes_only_when_transform_changes",
        ],
    );
    assert_contains_all(
        "native bitmap atlas mixed storage avoids stable source vectors",
        &text,
        &[
            "fn native_bitmap_atlas_renderer_storage_submissions<'a>(",
            "(!submission.submission.upload_commands().is_empty())",
            ".then(|| submission.source_bytes())",
            ".unwrap_or_default();",
        ],
    );
    assert_contains_all(
        "native bitmap atlas report aggregation remains a focused child owner",
        &prepare_report,
        &[
            "pub(super) fn glyph_atlas_bitmap_renderer_prepare_report_for_storage_passes(",
            "upload_plan_build_count",
            "upload_plan_skip_count",
        ],
    );
    assert_contains_all(
        "native bitmap atlas product stable-frame regression",
        &native_layout,
        &[
            "warm_prepare_report.upload_plan_build_count, 1",
            "stable_prepare_report.upload_plan_build_count, 0",
            "stable_prepare_report.upload_plan_skip_count, 1",
        ],
    );

    for (path, source) in [
        (
            "scene_renderer/ui/atlas_renderer/renderer.rs",
            renderer.as_str(),
        ),
        (
            "scene_renderer/ui/atlas_renderer/prepare_report.rs",
            prepare_report.as_str(),
        ),
        ("scene_renderer/ui/text.rs", text.as_str()),
    ] {
        assert!(
            source.lines().count() < 800,
            "{path} should stay under the R1.4 owner budget"
        );
    }
}
