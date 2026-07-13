use super::*;

#[test]
fn runtime_15_render_graph_resources_transient_aliasing_tests_are_child_owner() {
    let parent = read_runtime_src("render_graph/tests/resources.rs");
    let transient_aliasing = read_runtime_src("render_graph/tests/resources/transient_aliasing.rs");
    let plan_01 = read_repo(
        "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let builder_doc = read_repo("docs/zircon_runtime/render_graph/builder.md");
    let architecture_doc = read_repo("docs/assets-and-rendering/render-framework-architecture.md");

    assert_contains_all(
        "render graph resources parent keeps shared imports and child mount",
        &parent,
        &[
            "mod transient_aliasing;",
            "fn graph_tracks_transient_lifetimes_and_resource_edges(",
            "fn graph_records_attachment_clear_load_store_ops(",
            "fn graph_records_storage_writes_without_attachment_ops(",
            "fn graph_culling_keeps_manual_dependencies_of_live_passes(",
        ],
    );
    assert_contains_all(
        "transient aliasing child owns allocation plan contracts",
        &transient_aliasing,
        &[
            "use super::*;",
            "fn graph_builds_transient_aliasing_plan_for_non_overlapping_lifetimes(",
            "fn graph_transient_allocation_plan_reports_slot_reserved_bytes(",
            "plan.slot_bytes_for_bucket(",
            "plan.sparse_texture_virtual_bytes",
        ],
    );
    for moved_test in [
        "fn graph_builds_transient_aliasing_plan_for_non_overlapping_lifetimes(",
        "fn graph_transient_allocation_plan_reports_slot_reserved_bytes(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "render_graph/tests/resources.rs should mount the transient aliasing child instead of defining `{moved_test}`"
        );
        assert!(
            transient_aliasing.contains(moved_test),
            "render_graph/tests/resources/transient_aliasing.rs should own `{moved_test}`"
        );
    }

    for (path, source) in [
        ("render_graph/tests/resources.rs", parent.as_str()),
        (
            "render_graph/tests/resources/transient_aliasing.rs",
            transient_aliasing.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget after the transient aliasing split; got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 01", plan_01.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render graph builder doc", builder_doc.as_str()),
        (
            "render framework architecture doc",
            architecture_doc.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "RenderGraph resources transient aliasing tests owner split",
                "render_graph_resources_transient_aliasing_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "zircon_runtime/src/render_graph/tests/resources.rs",
                "zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs",
                "runtime_15_render_graph_resources_transient_aliasing_tests_are_child_owner",
            ],
        );
    }
}
