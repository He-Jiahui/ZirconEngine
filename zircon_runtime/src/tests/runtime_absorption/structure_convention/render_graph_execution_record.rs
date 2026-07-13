use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_render_graph_execution_record_is_folder_backed() {
    let parent = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs",
    );
    let compute_workload = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs",
    );
    let compute_workload_tests = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload/tests.rs",
    );
    let tests = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let plan_01 = format!(
        "{}\n{}",
        read_repo(
            "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
        ),
        render_index,
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let graph_execution_doc = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.md",
    );

    assert_contains_all(
        "execution record parent mounts focused child owners",
        &parent,
        &[
            "mod compute_workload;",
            "mod tests;",
            "pub use self::compute_workload::{",
            "pub struct RenderGraphLightGridReport",
            "pub struct RenderGraphExecutionRecord",
            "pub fn audit_compute_workload",
            "pub fn stage_execution_report",
        ],
    );
    for moved_owner in [
        "pub struct RenderGraphComputeDispatchRecord",
        "pub struct RenderGraphComputeWorkloadDispatchContext",
        "fn dispatch_groups_for_1d_group_count",
        "pub enum RenderGraphComputeWorkloadAuditStatus",
        "pub struct RenderGraphComputeWorkloadAuditRecord",
        "fn execution_record_tracks_compute_dispatch_metadata",
        "fn execution_record_preserves_resource_binding_report",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "render_graph_execution_record.rs should delegate {moved_owner} to child owners"
        );
    }
    assert_contains_all(
        "compute workload child owns compute dispatch/audit contracts",
        &compute_workload,
        &[
            "pub struct RenderGraphComputeDispatchRecord",
            "pub struct RenderGraphComputeWorkloadDispatchContext",
            "fn dispatch_groups_for_1d_group_count",
            "pub enum RenderGraphComputeWorkloadAuditStatus",
            "pub struct RenderGraphComputeWorkloadAuditRecord",
            "pub(super) fn matched_or_mismatched",
        ],
    );
    assert_contains_all(
        "compute workload tests child owns compute dispatch/audit coverage",
        &compute_workload_tests,
        &[
            "fn execution_record_tracks_compute_dispatch_metadata",
            "fn execution_record_audits_planned_compute_workloads_against_dispatches",
            "fn execution_record_flags_compute_workload_label_workgroup_and_extent_mismatches",
        ],
    );
    assert_contains_all(
        "execution record tests child owns non-compute record behavior",
        &tests,
        &[
            "fn execution_record_preserves_resource_binding_report",
            "fn execution_record_preserves_light_grid_report",
            "fn execution_record_counts_queue_lanes_from_executed_passes",
            "fn execution_record_counts_renderer_stage_order_violations",
            "fn execution_record_preserves_pass_debug_markers",
        ],
    );

    for (path, source, budget) in [
        (
            "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs",
            parent.as_str(),
            620,
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs",
            compute_workload.as_str(),
            680,
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs",
            tests.as_str(),
            430,
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload/tests.rs",
            compute_workload_tests.as_str(),
            500,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the owner budget {budget}; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 01", plan_01.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("graph execution doc", graph_execution_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 01 render graph execution record owner split",
                "render_plan01_execution_record_owner_split_static_passed",
                "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs",
                "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs",
                "runtime_15_render_graph_execution_record_is_folder_backed",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
