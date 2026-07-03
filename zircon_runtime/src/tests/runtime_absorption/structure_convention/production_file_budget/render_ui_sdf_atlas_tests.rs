use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_sdf_atlas_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/sdf_atlas.rs");
    let tests_mod = read_runtime_src("graphics/scene/scene_renderer/ui/sdf_atlas/tests/mod.rs");
    let plan = read_runtime_src("graphics/scene/scene_renderer/ui/sdf_atlas/tests/plan.rs");
    let allocation =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_atlas/tests/allocation.rs");
    let cache_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_atlas/tests/cache_report.rs");
    let owner = read_runtime_src("graphics/scene/scene_renderer/ui/sdf_atlas/tests/owner.rs");
    let tests = [
        tests_mod.as_str(),
        plan.as_str(),
        allocation.as_str(),
        cache_report.as_str(),
        owner.as_str(),
    ]
    .join("\n");

    let plan_14 = read_repo("docs/plans/zircon_runtime/render/14-2d-stack.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_submit = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let ui_text = read_repo("docs/zircon_runtime/ui/text.md");

    assert_contains_all(
        "screen-space UI SDF atlas parent keeps production owner and test mount",
        &parent,
        &[
            "pub(super) struct ScreenSpaceUiSdfAtlas",
            "pub(super) fn prepare(&mut self, texts: &[ScreenSpaceUiTextBatch])",
            "pub(super) fn plan(&self) -> &SdfAtlasPlan",
            "pub(super) fn cache_report(&self) -> SdfAtlasCacheReport",
            "pub(super) fn plan_sdf_atlas(texts: &[ScreenSpaceUiTextBatch]) -> SdfAtlasPlan",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn sdf_atlas_plan_deduplicates_glyph_slots_across_batches(",
        "fn sdf_atlas_plan_keys_glyph_slots_by_font_identity_and_fixed_bake_params(",
        "fn sdf_atlas_plan_preserves_whitespace_advances_without_slots(",
        "fn sdf_atlas_plan_assigns_slot_rects_by_key_not_batch_order(",
        "fn sdf_atlas_quality_controls_slot_size_and_min_grid(",
        "fn sdf_atlas_plan_uses_additional_pages_after_default_page_overflow(",
        "fn sdf_atlas_plan_reports_page_limit_allocation_failures(",
        "fn sdf_atlas_owner_retains_inactive_slots_between_non_empty_frames(",
        "fn sdf_atlas_owner_reports_retained_and_added_slots(",
        "fn sdf_atlas_owner_reuses_retained_slot_without_readding_glyph(",
        "fn sdf_atlas_owner_clears_previous_plan_for_native_only_frames(",
        "fn sdf_atlas_owner_preserves_whitespace_runs_without_cache_slots(",
        "fn sdf_atlas_owner_evicts_old_inactive_slots_when_cache_limit_is_exceeded(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "screen-space UI SDF atlas parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "screen-space UI SDF atlas test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "screen-space UI SDF atlas test owner keeps private helper coverage",
        &tests,
        &[
            "use super::*;",
            "ScreenSpaceUiSdfAtlas::new()",
            "SdfAtlasCacheReport",
            "SDF_ATLAS_MAX_CACHED_SLOT_COUNT",
            "fn text_batch(",
            "fn glyph_slots(",
            "fn glyph_range_string(",
        ],
    );

    for (path, source) in [
        ("scene_renderer/ui/sdf_atlas.rs", parent.as_str()),
        (
            "scene_renderer/ui/sdf_atlas/tests/mod.rs",
            tests_mod.as_str(),
        ),
        ("scene_renderer/ui/sdf_atlas/tests/plan.rs", plan.as_str()),
        (
            "scene_renderer/ui/sdf_atlas/tests/allocation.rs",
            allocation.as_str(),
        ),
        (
            "scene_renderer/ui/sdf_atlas/tests/cache_report.rs",
            cache_report.as_str(),
        ),
        ("scene_renderer/ui/sdf_atlas/tests/owner.rs", owner.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the SDF atlas test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 14", &plan_14),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("render product submit docs", &render_product_submit),
        ("UI text docs", &ui_text),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "Screen-space UI SDF atlas test owner split",
                "render_plan14_sdf_atlas_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "Runtime 15 M4 SDF atlas/render tests folder-backed guard sync",
                "runtime_15_sdf_atlas_render_tests_folder_backed_guard_sync_static_passed_cargo_deferred",
                "graphics/scene/scene_renderer/ui/sdf_atlas.rs",
                "graphics/scene/scene_renderer/ui/sdf_atlas/tests/mod.rs",
                "graphics/scene/scene_renderer/ui/sdf_atlas/tests/plan.rs",
                "graphics/scene/scene_renderer/ui/sdf_atlas/tests/allocation.rs",
                "graphics/scene/scene_renderer/ui/sdf_atlas/tests/owner.rs",
                "runtime_15_screen_space_ui_sdf_atlas_tests_are_child_owner_split",
            ],
        );
    }
}
