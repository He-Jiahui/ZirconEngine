use super::{assert_contains_all, read_runtime_src};
use crate::tests::runtime_absorption::structure_convention::runtime_src_path;

#[test]
fn runtime_15_screen_space_ui_text_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let resolved_batches =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/resolved_batches.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/text/tests.rs");
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

    for moved_test in [
        "fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches(",
        "fn text_backend_routing_respects_auto_font_mode_without_crossing_backends(",
        "fn text_prepare_report_summarizes_input_routing_and_sdf_reports(",
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
            tests.contains(moved_test),
            "screen-space UI text test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "screen-space UI text test owner keeps private helper coverage",
        &tests,
        &[
            "use super::super::sdf_atlas::SdfAtlasDirtyPageReport;",
            "use super::super::sdf_upload::SdfAtlasUploadPageReport;",
            "use crate::text::sdf::SdfAtlasRect;",
            "use super::*;",
            "fn text_batch(",
            "native_text_area_placement(",
        ],
    );
    assert_eq!(
        tests.matches("#[test]").count(),
        14,
        "screen-space UI text test owner should preserve the current 14 private regression tests"
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
