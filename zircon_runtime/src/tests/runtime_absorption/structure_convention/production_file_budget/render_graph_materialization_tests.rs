use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_graph_materialization_tests_are_child_owner_split() {
    let parent =
        read_runtime_src("graphics/scene/scene_renderer/graph_execution/materialization.rs");
    let tests =
        read_runtime_src("graphics/scene/scene_renderer/graph_execution/materialization/tests.rs");

    let plan_01 = read_repo("docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let transient_materialization_docs = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/transient_materialization.md",
    );
    let execution_resources_docs = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.md",
    );

    assert_contains_all(
        "RenderGraph materialization parent keeps production owner and test mount",
        &parent,
        &[
            "pub(super) fn materialize_transient_resources(",
            "pub(super) fn create_wgpu_texture(",
            "pub(super) fn create_wgpu_buffer(",
            "fn wgpu_texture_usages(",
            "fn wgpu_buffer_usages(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn non_storage_texture_formats_do_not_request_storage_binding(",
        "fn storage_texture_formats_request_storage_binding(",
        "fn materialization_creates_dense_transients_and_skips_sparse_reservations(",
        "fn materialization_aliases_compatible_transient_texture_slots(",
        "fn materialization_receives_incompatible_texture_resources_in_separate_graph_slots(",
        "fn materialization_overrides_preimported_terminal_aa_input_with_owned_transient(",
        "fn materialization_aliases_transient_buffer_slots(",
        "fn materialization_exposes_owned_texture_mip_views(",
        "fn materialization_aliases_ssr_reflection_coarse_pyramid_to_parent_mip_view(",
        "fn materialization_allocates_ssr_reflection_coarse_resource_when_parent_has_no_coarse_mip(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "RenderGraph materialization parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "RenderGraph materialization test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "RenderGraph materialization tests keep WGPU resource fixtures",
        &tests,
        &[
            "use super::*;",
            "RenderBackend::new_offscreen()",
            "storage_requested_usages_for(",
            "texture_alias_for(",
            "buffer_alias_for(",
        ],
    );

    for (path, source) in [
        ("graph_execution/materialization.rs", parent.as_str()),
        ("graph_execution/materialization/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the materialization test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 01", &plan_01),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        (
            "transient materialization docs",
            &transient_materialization_docs,
        ),
        ("execution resources docs", &execution_resources_docs),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "RenderGraph materialization test owner split",
                "render_plan01_materialization_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/graph_execution/materialization.rs",
                "graphics/scene/scene_renderer/graph_execution/materialization/tests.rs",
                "runtime_15_render_graph_materialization_tests_are_child_owner_split",
            ],
        );
    }
}
