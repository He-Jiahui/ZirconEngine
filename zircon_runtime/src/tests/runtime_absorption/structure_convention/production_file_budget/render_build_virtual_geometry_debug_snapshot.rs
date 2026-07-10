use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_vg_debug_snapshot_is_child_owner_split() {
    let parent = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs",
    );
    let page = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/page.rs",
    );
    let node_cull = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/node_cull.rs",
    );
    let execution = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs",
    );
    let support = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/support.rs",
    );

    let plan_02 = read_repo("docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let visibility_doc = read_repo("docs/zircon_runtime/graphics/visibility.md");
    let module_doc = read_repo(
        "docs/zircon_runtime/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.md",
    );

    assert_contains_all(
        "VG debug snapshot parent stays a thin orchestration owner",
        &parent,
        &[
            "mod execution;",
            "mod node_cull;",
            "mod page;",
            "mod support;",
            "pub(super) fn build_virtual_geometry_debug_snapshot(",
            "build_cull_input_snapshot(",
            "build_node_and_cluster_cull_snapshot(",
            "build_execution_snapshot(",
            "RenderVirtualGeometryDebugSnapshot {",
        ],
    );

    for moved_owner in [
        "fn build_resident_page_inspections(",
        "fn build_cull_input_snapshot(",
        "struct NodeAndClusterCullSnapshot",
        "fn build_node_and_cluster_cull_snapshot(",
        "fn build_node_and_cluster_cull_traversal_records(",
        "struct ExecutionSnapshot",
        "fn build_execution_snapshot(",
        "fn build_selected_clusters_from_execution_segments(",
        "fn build_hardware_rasterization_records_from_execution_segments(",
        "fn saturated_u32_len(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "VG debug snapshot parent should delegate instead of owning {moved_owner}"
        );
    }

    assert_contains_all(
        "page owner keeps residency, request, and cull input projections",
        &page,
        &[
            "pub(super) fn build_cull_input_snapshot(",
            "fn unique_extract_entity_count(",
            "pub(super) fn build_resident_page_inspections(",
            "pub(super) fn build_available_page_slots(",
            "pub(super) fn build_pending_page_request_inspections(",
            "pub(super) fn build_evictable_page_inspections(",
            "fn page_size_bytes(",
        ],
    );
    assert_contains_all(
        "node cull owner keeps GPU-style hierarchy traversal replay",
        &node_cull,
        &[
            "pub(super) struct NodeAndClusterCullSnapshot",
            "pub(super) fn build_node_and_cluster_cull_snapshot(",
            "fn build_node_and_cluster_cull_global_state(",
            "fn build_node_and_cluster_cull_instance_seeds(",
            "fn build_node_and_cluster_cull_traversal_records(",
            "fn build_node_and_cluster_cull_page_request_ids(",
            "struct TraversalQueueItem",
        ],
    );
    assert_contains_all(
        "execution owner keeps draw segment, selected cluster, and visbuffer evidence",
        &execution,
        &[
            "pub(super) struct ExecutionSnapshot",
            "pub(super) fn build_execution_snapshot(",
            "fn execution_state_for_page(",
            "pub(super) fn build_selected_clusters_from_execution_segments(",
            "pub(super) fn build_visbuffer_debug_marks_from_selected_clusters(",
            "pub(super) fn build_visbuffer64_entries_from_selected_clusters(",
            "pub(super) fn build_hardware_rasterization_records_from_execution_segments(",
        ],
    );
    assert_contains_all(
        "support owner keeps shared saturation helper",
        &support,
        &["pub(super) fn saturated_u32_len("],
    );

    for (path, source) in [
        (
            "submit/build_virtual_geometry_debug_snapshot.rs",
            parent.as_str(),
        ),
        (
            "submit/build_virtual_geometry_debug_snapshot/page.rs",
            page.as_str(),
        ),
        (
            "submit/build_virtual_geometry_debug_snapshot/node_cull.rs",
            node_cull.as_str(),
        ),
        (
            "submit/build_virtual_geometry_debug_snapshot/execution.rs",
            execution.as_str(),
        ),
        (
            "submit/build_virtual_geometry_debug_snapshot/support.rs",
            support.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 production owner budget, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 02", &plan_02),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("visibility docs", &visibility_doc),
        ("VG debug snapshot docs", &module_doc),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "VG debug snapshot owner split",
                "render_plan02_vg_debug_snapshot_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs",
                "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/page.rs",
                "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/node_cull.rs",
                "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs",
                "runtime_15_render_vg_debug_snapshot_is_child_owner_split",
            ],
        );
    }
}
