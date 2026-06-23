use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_stats_graph_execution_resources_are_child_owner() {
    let parent = read_runtime_src("core/runtime/diagnostics/render_stats_store/graph.rs");
    let execution_resources = read_runtime_src(
        "core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let diagnostics_doc = read_repo("docs/zircon_runtime/core/diagnostics.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "render-stats graph parent keeps graph dispatcher responsibilities",
        &parent,
        &[
            "mod execution_resources;",
            "pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats)",
            "record_frame_graph(store, stats);",
            "execution_resources::record(store, frame_index, stats);",
            "record_graph_materialization(store, frame_index, stats);",
            "record_post_process_graph(store, stats);",
        ],
    );
    for moved_owner in [
        "fn record_graph_execution_resources",
        "render.graph.execution.texture_view_count",
        "render.graph.execution.transient_pool.texture_created_count",
        "render.graph.execution.transient_pool.buffer_pool_budget_bytes",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "render_stats_store/graph.rs should delegate {moved_owner} to graph/execution_resources.rs"
        );
    }
    assert_contains_all(
        "execution resources child owns transient-pool and binding diagnostics",
        &execution_resources,
        &[
            "pub(super) fn record",
            "stats.last_graph_execution_resource_report",
            "render.graph.execution.texture_view_count",
            "render.graph.execution.transient_pool.texture_created_count",
            "render.graph.execution.transient_pool.buffer_pool_budget_bytes",
            "record_bytes(",
        ],
    );

    for (path, source) in [
        (
            "core/runtime/diagnostics/render_stats_store/graph.rs",
            parent.as_str(),
        ),
        (
            "core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs",
            execution_resources.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("core diagnostics doc", diagnostics_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 core runtime render-stats graph execution-resources owner split",
                "runtime_15_render_stats_graph_execution_resources_owner_split_static_passed_cargo_timeout_no_result",
                "core/runtime/diagnostics/render_stats_store/graph.rs",
                "core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs",
                "runtime_15_render_stats_graph_execution_resources_are_child_owner",
            ],
        );
    }
}
