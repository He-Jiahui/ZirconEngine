use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_text_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/text/tests.rs");

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let text_plan =
        read_repo("docs/plans/zircon_runtime/text/09/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let graphics_text_doc = read_repo("docs/zircon_runtime/graphics/text.md");
    let ui_text_doc = read_repo("docs/zircon_runtime/ui/text.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_text_template.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "screen-space UI text parent keeps production owner and test mount",
        &parent,
        &[
            "mod font_id_report;",
            "mod sdf_fallback;",
            "pub(super) struct ScreenSpaceUiTextSystem",
            "fn resolve_text_batches(",
            "fn native_text_area_placement(",
            "fn native_text_align(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches(",
        "fn text_backend_routing_respects_auto_font_mode_without_crossing_backends(",
        "fn text_prepare_report_summarizes_input_routing_and_sdf_reports(",
        "fn auto_text_mode_uses_font_asset_default_when_present(",
        "fn explicit_text_mode_overrides_font_asset_default(",
        "fn auto_text_mode_falls_back_to_native_without_font_asset_default(",
        "fn text_attrs_maps_shared_rich_run_style_to_glyphon_attrs(",
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
            "use super::super::sdf_atlas::{SdfAtlasDirtyPageReport, SdfAtlasRect};",
            "use super::super::sdf_upload::SdfAtlasUploadPageReport;",
            "use super::*;",
            "fn text_batch(",
            "native_text_area_placement(",
        ],
    );
    assert_eq!(
        tests.matches("#[test]").count(),
        12,
        "screen-space UI text test owner should preserve the current 12 private regression tests"
    );

    for (path, source) in [
        ("scene_renderer/ui/text.rs", parent.as_str()),
        ("scene_renderer/ui/text/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the Runtime 15 owner budget after the test split, got {line_count}"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Runtime text plan", text_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("graphics text doc", graphics_text_doc.as_str()),
        ("UI text doc", ui_text_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("status map", status_map.as_str()),
        ("date map", date_map.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 screen-space UI text tests owner split",
                "runtime_15_screen_space_ui_text_tests_owner_split_static_passed_cargo_deferred",
                "graphics/scene/scene_renderer/ui/text.rs",
                "graphics/scene/scene_renderer/ui/text/tests.rs",
                "runtime_15_screen_space_ui_text_tests_are_child_owner_split",
            ],
        );
    }
}
