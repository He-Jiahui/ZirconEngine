#[test]
fn runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit() {
    let large_file_doc =
        include_str!("../../../../../../docs/engine-architecture/large-file-ownership-m1.md");
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let architecture_review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let interface_doc =
        include_str!("../../../../../../docs/engine-architecture/runtime-interface-convergence.md");

    for required_large_file_doc_anchor in [
        "`classified-and-clear`",
        "`hotspot_count = 0`",
        "`classification_count = 0`",
        "`decision_group_count = 0`",
        "`large_file_migration_debt_count = 0`",
        "`unclassified_hotspot_count = 0`",
        "`unclassified_hotspots = []`",
    ] {
        assert!(
            large_file_doc.contains(required_large_file_doc_anchor),
            "large-file owner gate doc should retain current audit anchor `{required_large_file_doc_anchor}`"
        );
    }

    for stale_large_file_doc_anchor in [
        "zircon_hub/src/app/runtime.rs",
        "zircon_hub/src/app/view_model.rs",
        "`hotspot_count = 25`",
        "`classification_count = 5`",
        "`decision_group_count = 5`",
        "`large_file_migration_debt_count = 5`",
        "`hotspot_count = 33`",
        "`hotspot_count = 30`",
        "`hotspot_count = 40`",
        "`hotspot_count = 42`",
        "`hotspot_count = 41`",
        "`hotspot_count = 39`",
        "`hotspot_count = 38`",
        "`hotspot_count = 37`",
        "`hotspot_count = 36`",
        "`editor-retained-host = 11`",
        "`editor-retained-host = 10`",
        "`runtime-framework-render = 1`",
        "`runtime-framework-render = 2`",
        "`runtime-framework-render = 4`",
        "`runtime-other = 10`",
        "`runtime-other = 18`",
        "`editor-retained-host = 12`",
        "`runtime-other = 17`",
        "`runtime-other = 16`",
        "`runtime-other = 15`",
        "`runtime-other = 14`",
        "`runtime-other = 12`",
        "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs",
        "zircon_runtime/src/asset/assets/scene.rs",
        "zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs`: 1495 lines",
    ] {
        assert!(
            !large_file_doc.contains(stale_large_file_doc_anchor),
            "large-file owner gate doc should not keep stale audit anchor `{stale_large_file_doc_anchor}`"
        );
    }

    for required_runtime_07_owner_gate_anchor in [
        "Runtime 07 owner-budgeted optimization gate",
        "large_file_ownership_gate",
        "classified-and-clear",
        "hotspots 0",
        "debt groups 0",
        "owner classes 0",
        "unclassified 0",
    ] {
        assert!(
            runtime_07_plan.contains(required_runtime_07_owner_gate_anchor)
                || runtime_index.contains(required_runtime_07_owner_gate_anchor)
                || hotspot_doc.contains(required_runtime_07_owner_gate_anchor)
                || architecture_review.contains(required_runtime_07_owner_gate_anchor)
                || interface_doc.contains(required_runtime_07_owner_gate_anchor),
            "Runtime 07 owner-budget gate mirrors should retain `{required_runtime_07_owner_gate_anchor}`"
        );
    }

    for required_mirror_anchor in [
        "hotspots 0, debt groups 0, owner classes 0, unclassified hotspots 0",
        "0 hotspots, 0 migration-debt owner groups, and zero unclassified hotspots",
        "no current owner-class buckets above the large-file threshold",
        "threshold 1000 lines, 0 hotspots, 0 owner debt groups, 0 owner classes, and 0 unclassified hotspots",
    ] {
        assert!(
            runtime_07_plan.contains(required_mirror_anchor)
                || runtime_index.contains(required_mirror_anchor)
                || hotspot_doc.contains(required_mirror_anchor)
                || architecture_review.contains(required_mirror_anchor)
                || interface_doc.contains(required_mirror_anchor),
            "Runtime 07 mirror docs should retain exact large-file gate summary `{required_mirror_anchor}`"
        );
    }

    assert!(
        hotspot_doc.contains(
            "threshold 1000 lines, 0 hotspots, 0 owner debt groups, 0 owner classes, and 0 unclassified hotspots"
        ),
        "hotspot inventory should carry the current 0-hotspot owner-budget gate, not only historical drift rows"
    );
    assert!(
        architecture_review.contains(
            "reports M1 gate status `classified-and-clear`, 0 hotspots, 0 migration-debt owner groups, and zero unclassified hotspots"
        ),
        "architecture review should carry the current 0-hotspot large-file audit summary"
    );
    assert!(
        interface_doc.contains(
            "current audit output reports 0 hotspots above 1000 lines, M1 gate status `classified-and-clear`, 0 migration-debt owner groups, and zero unclassified hotspots"
        ),
        "interface convergence doc should carry the current 0-hotspot large-file audit summary"
    );
}
