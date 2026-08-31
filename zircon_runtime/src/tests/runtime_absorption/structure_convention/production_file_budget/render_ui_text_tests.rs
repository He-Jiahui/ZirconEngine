use super::{assert_contains_all, read_runtime_src};
use crate::tests::runtime_absorption::structure_convention::runtime_src_path;

#[test]
fn runtime_15_screen_space_ui_text_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let resolved_batches =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/resolved_batches.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/text/tests.rs");
    let font_asset_tests =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/tests/font_assets.rs");
    let prepare_report_tests =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/tests/prepare_report.rs");
    let rendering_tests =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/tests/rendering.rs");
    let test_support = read_runtime_src("graphics/scene/scene_renderer/ui/text/tests/support.rs");
    let native_glyph_run =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/native_glyph_run.rs");
    let native_bitmap_atlas = read_runtime_src("text/native_bitmap_atlas.rs");
    let native_bitmap_handoff = read_runtime_src("text/native_bitmap_atlas/handoff.rs");
    let render_state = read_runtime_src("text/render_state.rs");
    let native_bitmap_atlas_tests = read_runtime_src("text/native_bitmap_atlas/tests.rs");
    let native_bitmap_source_cache_tests =
        read_runtime_src("text/native_bitmap_atlas/tests/source_cache.rs");
    let native_bitmap_source_cache_worker_request_tests =
        read_runtime_src("text/native_bitmap_atlas/tests/source_cache/worker_requests.rs");

    assert_contains_all(
        "screen-space UI text parent keeps production owner and test mount",
        &parent,
        &[
            "mod font_id_report;",
            "mod native_glyph_run;",
            "mod resolved_batches;",
            "mod sdf_fallback;",
            "pub(super) struct ScreenSpaceUiTextSystem",
            "use self::resolved_batches::resolve_text_batches;",
            "use self::native_glyph_run::native_bitmap_atlas_glyph_runs;",
            "struct ScreenSpaceUiTextBackend;",
            "#[cfg(test)]\nmod tests;",
        ],
    );
    assert_contains_all(
        "screen-space UI text native glyph-run owner projects canonical shaped glyphs",
        &native_glyph_run,
        &[
            "pub(in crate::graphics::scene::scene_renderer::ui) fn native_bitmap_atlas_glyph_runs(",
            "pub(in crate::graphics::scene::scene_renderer::ui) struct NativeBitmapAtlasGlyphRunProjection",
            "resolve_font_handle_batch(",
            "GlyphRasterKey::from_request(",
            "glyph_artifact_line",
            "text.text_decoration_baseline",
        ],
    );
    for forbidden_legacy_input in [
        "TextArea",
        "TextRenderer",
        "TextAtlas",
        "layout_runs()",
        "shape_native_buffer",
    ] {
        assert!(
            !parent.contains(forbidden_legacy_input),
            "screen-space UI text must not restore legacy native input `{forbidden_legacy_input}`"
        );
    }
    assert_contains_all(
        "native bitmap atlas reports its own readiness and degradation state",
        &native_bitmap_handoff,
        &[
            "NativeBitmapAtlasDegradationReason",
            "NativeBitmapAtlasHandoff::Degraded",
            "report.native_submission_ready",
        ],
    );
    for forbidden_legacy_handoff in [
        "GlyphonFallback",
        "replaces_glyphon",
        "glyphon_fallback_reason",
        "native_bitmap_atlas_glyphon_fallback_reason_for_report",
    ] {
        assert!(
            !native_bitmap_atlas.contains(forbidden_legacy_handoff)
                && !native_bitmap_handoff.contains(forbidden_legacy_handoff),
            "native bitmap atlas must not restore legacy handoff `{forbidden_legacy_handoff}`"
        );
    }
    assert_contains_all(
        "screen-space UI text resolved-batches child owns batch resolution",
        &resolved_batches,
        &["pub(super) fn resolve_text_batches_after_font_dependencies("],
    );

    assert_contains_all(
        "screen-space UI text test facade remains folder-backed",
        &tests,
        &[
            "#[path = \"tests/font_assets.rs\"]\nmod font_assets;",
            "#[path = \"tests/prepare_report.rs\"]\nmod prepare_report;",
            "#[path = \"tests/rendering.rs\"]\nmod rendering;",
            "#[path = \"tests/support.rs\"]\nmod support;",
        ],
    );

    for moved_test in [
        "fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches(",
        "fn text_backend_routing_respects_auto_font_mode_without_crossing_backends(",
        "fn auto_text_mode_uses_font_asset_default_when_present(",
        "fn explicit_text_mode_overrides_font_asset_default(",
        "fn auto_text_mode_falls_back_to_native_without_font_asset_default(",
        "fn native_text_backend_accepts_only_prepared_glyph_runs(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "screen-space UI text parent should not own moved test `{moved_test}`"
        );
        assert!(
            rendering_tests.contains(moved_test),
            "screen-space UI text rendering owner should contain moved test `{moved_test}`"
        );
    }

    for moved_test in [
        "fn text_prepare_report_summarizes_input_routing_and_sdf_reports(",
        "fn text_prepare_report_exposes_raster_upload_scroll_counters(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "screen-space UI text parent should not own moved test `{moved_test}`"
        );
        assert!(
            prepare_report_tests.contains(moved_test),
            "screen-space UI text prepare-report owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "screen-space UI text folder-backed owners keep private helper coverage",
        &rendering_tests,
        &[
            "native_text_backend_accepts_only_prepared_glyph_runs(",
            "text_batch(\"Normal\", UiTextRenderMode::Native)",
        ],
    );
    assert!(
        (font_asset_tests.matches("#[test]").count()
            + prepare_report_tests.matches("#[test]").count()
            + rendering_tests.matches("#[test]").count())
            >= 23,
        "screen-space UI text folder-backed owners should preserve at least 23 private regression tests"
    );
    assert_contains_all(
        "screen-space UI text folder-backed fixture owner",
        &test_support,
        &[
            "pub(super) fn text_batch(",
            "pub(super) struct TextFontProject",
            "pub(super) struct RuntimeFontAssetGuard",
        ],
    );
    assert_contains_all(
        "screen-space UI text prepare-report owner keeps its shared fixture import",
        &prepare_report_tests,
        &[
            "use super::support::text_batch;",
            "use crate::text::sdf::SdfAtlasRect;",
            "use super::super::super::sdf_atlas::SdfAtlasDirtyPageReport;",
            "use super::super::super::sdf_upload::SdfAtlasUploadPageReport;",
        ],
    );
    assert_contains_all(
        "text render state owns bitmap atlas frame index regression coverage",
        &render_state,
        &[
            "fn advance_bitmap_atlas_frame_index(",
            "fn bitmap_atlas_frame_index_advances_monotonically_and_saturates(",
        ],
    );
    assert_contains_all(
        "text native bitmap atlas owns its test modules",
        &native_bitmap_atlas_tests,
        &[
            "mod frame_tests;",
            "mod handoff_tests;",
            "mod retry_frame_tests;",
            "mod source_cache_tests;",
            "mod source_tests;",
        ],
    );
    assert_contains_all(
        "native bitmap source-cache tests keep worker requests in a child owner",
        &native_bitmap_source_cache_tests,
        &[
            "#[path = \"source_cache/worker_requests.rs\"]",
            "mod worker_request_tests;",
        ],
    );
    for moved_test in [
        "fn native_bitmap_atlas_source_cache_requests_exact_instance_once_per_glyph(",
        "fn native_bitmap_atlas_source_cache_reports_backpressure_without_marking_work_pending(",
        "fn native_bitmap_atlas_source_cache_reloads_font_bytes_after_face_invalidation(",
    ] {
        assert!(
            !native_bitmap_source_cache_tests.contains(moved_test),
            "native bitmap source-cache parent should not own moved test `{moved_test}`"
        );
        assert!(
            native_bitmap_source_cache_worker_request_tests.contains(moved_test),
            "native bitmap source-cache worker-request owner should contain moved test `{moved_test}`"
        );
    }
    assert!(
        !runtime_src_path("graphics/scene/scene_renderer/ui/text/tests/native_bitmap_atlas.rs")
            .exists(),
        "screen-space UI text should not retain the retired bitmap-atlas frame-index test owner"
    );
}
