use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_shadow_atlas_plan_tests_are_child_owners() {
    let allocator = read_runtime_src("graphics/scene/scene_renderer/shadow/atlas/allocator.rs");
    let allocator_tests =
        read_runtime_src("graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs");
    let plan = read_runtime_src("graphics/scene/scene_renderer/shadow/plan.rs");
    let plan_tests = read_runtime_src("graphics/scene/scene_renderer/shadow/plan/tests.rs");

    let plan_05 = read_repo("docs/plans/zircon_runtime/render/05/2026-07-09-lighting-shadows-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let shadow_doc = read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/shadow.md");

    assert_contains_all(
        "shadow atlas allocator keeps production allocation owner and child test mount",
        &allocator,
        &[
            "pub(crate) struct ShadowAtlasAllocator",
            "pub(crate) fn allocate_frame(",
            "struct FreeRectPacker",
            "fn compact_free_rects(",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );
    assert_contains_all(
        "shadow frame plan keeps production plan owner and child test mount",
        &plan,
        &[
            "pub(crate) struct ShadowFramePlan",
            "pub(crate) fn build_shadow_frame_plan(",
            "fn append_directional_cascades(",
            "fn append_point_light_slots(",
            "fn append_spot_light_slots(",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );

    for moved_test in [
        "fn render_shadow_atlas_allocates_tiers_descending(",
        "fn render_shadow_atlas_global_downgrade_fits_pressure(",
        "fn render_shadow_atlas_evicts_lowest_priority_on_pressure(",
        "fn render_shadow_atlas_hysteresis_prevents_flapping(",
        "fn render_shadow_atlas_preempts_after_confirmed_priority_margin(",
        "fn render_shadow_atlas_scale_bias_matches_slice_transform(",
    ] {
        assert!(
            !allocator.contains(moved_test),
            "shadow atlas allocator production owner should delegate moved test `{moved_test}`"
        );
    }
    for moved_test in [
        "fn render_shadow_frame_plan_assigns_first_directional_cascade_slots(",
        "fn render_shadow_frame_plan_builds_distinct_directional_cascade_matrices(",
        "fn render_shadow_frame_plan_caps_directional_cascade_tier_to_atlas_row(",
        "fn render_shadow_frame_plan_assigns_point_light_contiguous_face_slots(",
        "fn render_shadow_frame_plan_assigns_spot_light_slot_view_key(",
        "fn render_shadow_frame_plan_encodes_per_light_pcf_quality(",
        "fn render_shadow_light_slot_assignments_patch_packed_light_contract(",
    ] {
        assert!(
            !plan.contains(moved_test),
            "shadow frame plan production owner should delegate moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "shadow atlas allocator tests own allocation pressure coverage",
        &allocator_tests,
        &[
            "use super::*;",
            "fn render_shadow_atlas_allocates_tiers_descending(",
            "fn render_shadow_atlas_global_downgrade_fits_pressure(",
            "fn render_shadow_atlas_evicts_lowest_priority_on_pressure(",
            "fn render_shadow_atlas_hysteresis_prevents_flapping(",
            "fn render_shadow_atlas_preempts_after_confirmed_priority_margin(",
            "fn render_shadow_atlas_scale_bias_matches_slice_transform(",
        ],
    );
    assert_contains_all(
        "shadow frame plan tests own cascade, point, spot, pcf, and light-slot coverage",
        &plan_tests,
        &[
            "use super::*;",
            "fn render_shadow_frame_plan_assigns_first_directional_cascade_slots(",
            "fn render_shadow_frame_plan_builds_distinct_directional_cascade_matrices(",
            "fn render_shadow_frame_plan_caps_directional_cascade_tier_to_atlas_row(",
            "fn render_shadow_frame_plan_assigns_point_light_contiguous_face_slots(",
            "fn render_shadow_frame_plan_assigns_spot_light_slot_view_key(",
            "fn render_shadow_frame_plan_encodes_per_light_pcf_quality(",
            "fn render_shadow_light_slot_assignments_patch_packed_light_contract(",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/scene_renderer/shadow/atlas/allocator.rs",
            allocator.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs",
            allocator_tests.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/shadow/plan.rs",
            plan.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/shadow/plan/tests.rs",
            plan_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4/R4.3 owner budget after the shadow atlas/plan test split, got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 05", plan_05.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("shadow module docs", shadow_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 05 shadow atlas/plan test owner split",
                "render_plan05_shadow_atlas_plan_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/shadow/atlas/allocator.rs",
                "graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs",
                "graphics/scene/scene_renderer/shadow/plan.rs",
                "graphics/scene/scene_renderer/shadow/plan/tests.rs",
                "runtime_15_shadow_atlas_plan_tests_are_child_owners",
            ],
        );
    }
}
