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
    let native_buffer = read_runtime_src("text/native_buffer.rs");
    let render_state = read_runtime_src("text/render_state.rs");
    let native_bitmap_atlas_tests = read_runtime_src("text/native_bitmap_atlas/tests.rs");

    assert_contains_all(
        "screen-space UI text parent keeps production owner and test mount",
        &parent,
        &[
            "mod font_id_report;",
            "mod resolved_batches;",
            "mod sdf_fallback;",
            "pub(super) struct ScreenSpaceUiTextSystem",
            "use self::resolved_batches::resolve_text_batches;",
            "fn native_text_area_placement(",
            "fn native_text_align(",
            "#[cfg(test)]\nmod tests;",
        ],
    );
    assert_contains_all(
        "screen-space UI text resolved-batches child owns batch resolution",
        &resolved_batches,
        &["pub(super) fn resolve_text_batches("],
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
        "fn native_text_align_maps_start_end_through_text_direction(",
        "fn native_text_area_placement_snaps_fractional_origin_to_device_pixels(",
        "fn native_text_area_placement_drops_non_finite_origin_values(",
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
            "native_text_area_placement(",
            "text_batch(\"Normal\", UiTextRenderMode::Native)",
        ],
    );
    assert_eq!(
        font_asset_tests.matches("#[test]").count()
            + prepare_report_tests.matches("#[test]").count()
            + rendering_tests.matches("#[test]").count(),
        23,
        "screen-space UI text folder-backed owners should preserve the current 23 private regression tests"
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
        "text CPU preparation owns native attrs regression coverage",
        &native_buffer,
        &["fn native_attrs_are_owned_by_text_cpu_preparation("],
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
    assert!(
        !runtime_src_path("graphics/scene/scene_renderer/ui/text/tests/native_bitmap_atlas.rs")
            .exists(),
        "screen-space UI text should not retain the retired bitmap-atlas frame-index test owner"
    );
}
