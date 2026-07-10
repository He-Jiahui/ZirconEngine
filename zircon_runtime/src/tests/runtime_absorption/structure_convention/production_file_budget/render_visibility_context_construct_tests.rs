use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_visibility_context_construct_tests_are_child_owner() {
    let parent =
        read_runtime_src("graphics/visibility/context/from_extract_with_history/construct.rs");
    let tests = read_runtime_src(
        "graphics/visibility/context/from_extract_with_history/construct/tests.rs",
    );

    let plan_04 = read_repo("docs/plans/zircon_runtime/render/04/2026-07-09-visibility-culling-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let visibility_doc = read_repo("docs/zircon_runtime/graphics/visibility.md");
    let shadow_doc = read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/shadow.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");

    assert_contains_all(
        "VisibilityContext construct parent keeps frame construction, static index prefilter, and child test mount",
        &parent,
        &[
            "pub fn from_extract_with_history(",
            "pub(crate) fn from_extract_with_history_and_static_index(",
            "pub(crate) fn from_extract_with_history_static_index_and_task_pool(",
            "fn static_bvh_instances(",
            "fn build_static_index(",
            "fn cull_main_view_with_static_index(",
            "fn static_index_prefilter_candidates(",
            "fn conservative_camera_query_bounds(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_anchor in [
        "fn visibility_context_records_relevance_and_filters_main_view_layers(",
        "fn visibility_batch_key_preserves_layers_above_legacy_mask_width(",
        "fn visibility_context_builds_shadow_view_independent_from_main_layers(",
        "fn visibility_context_builds_shadow_views_for_atlas_light_slots(",
        "fn visibility_context_builds_custom_target_view_from_camera_descriptors(",
        "fn visibility_context_reuses_static_index_without_frame_rebuild(",
        "fn visibility_context_rebuilds_static_index_when_previous_index_is_missing(",
        "fn visibility_context_uses_static_index_prefilter_above_threshold(",
        "fn camera_descriptor_with_layers(",
        "fn frame_from_meshes(",
        "fn mesh_at(",
        "fn shadow_settings(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "VisibilityContext construct parent should delegate `{moved_anchor}` to tests.rs"
        );
        assert!(
            tests.contains(moved_anchor),
            "VisibilityContext construct test owner should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "VisibilityContext construct test owner keeps relevance, shadow, custom-target, and static-index coverage",
        &tests,
        &[
            "use crate::core::framework::render::{",
            "VisibilityContext::from_extract(&frame)",
            "VisibilityViewKey::ShadowCascade",
            "VisibilityViewKey::CustomTarget",
            "super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES",
        ],
    );

    for (path, source) in [
        (
            "visibility/context/from_extract_with_history/construct.rs",
            parent.as_str(),
        ),
        (
            "visibility/context/from_extract_with_history/construct/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 04", &plan_04),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("visibility docs", &visibility_doc),
        ("shadow docs", &shadow_doc),
        ("mesh pass docs", &mesh_pass_doc),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "VisibilityContext construct tests owner split",
                "render_plan04_visibility_context_construct_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/visibility/context/from_extract_with_history/construct.rs",
                "graphics/visibility/context/from_extract_with_history/construct/tests.rs",
                "runtime_15_visibility_context_construct_tests_are_child_owner",
            ],
        );
    }
}
