#[test]
fn runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit() {
    let large_file_doc =
        include_str!("../../../../../docs/engine-architecture/large-file-ownership-m1.md");
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let architecture_review =
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let interface_doc =
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");

    for required_large_file_doc_anchor in [
        "`hotspot_count = 30`",
        "`classification_count = 5`",
        "`decision_group_count = 5`",
        "`large_file_migration_debt_count = 5`",
        "`unclassified_hotspot_count = 0`",
        "`editor-retained-host = 3`",
        "`editor-ui = 8`",
        "`runtime-framework-render = 3`",
        "`runtime-other = 13`",
        "`support-hub = 3`",
        "zircon_runtime/src/asset/assets/scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs",
        "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs",
        "zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/{bvh_visualization,cpu_reference,cull_input,execution,node_and_cluster_cull,snapshot,sources}.rs",
        "zircon_runtime/src/navigation/runtime/{baked_mesh,world_scan,avoidance,state,math,tests}.rs",
        "zircon_runtime/src/core/framework/render/backend_types.rs",
        "zircon_runtime/src/core/framework/render/post_process/stack.rs",
        "zircon_runtime/src/core/framework/render/post_process/volume_component.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
        "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs",
        "zircon_hub/src/tauri_app/runtime_state/project_actions.rs",
        "zircon_hub/src/tauri_app/view_model.rs",
        "zircon_hub/src/tauri_app/runtime_state.rs",
    ] {
        assert!(
            large_file_doc.contains(required_large_file_doc_anchor),
            "large-file owner gate doc should retain current audit anchor `{required_large_file_doc_anchor}`"
        );
    }

    for stale_large_file_doc_anchor in [
        "zircon_hub/src/app/runtime.rs",
        "zircon_hub/src/app/view_model.rs",
        "`hotspot_count = 33`",
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
        "migration-debt-present",
        "hotspots 30",
        "debt groups 5",
        "owner classes 5",
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
        "hotspots 30, debt groups 5, owner classes 5, unclassified hotspots 0",
        "30 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots",
        "`editor-retained-host=3`, `editor-ui=8`, `runtime-framework-render=3`, `runtime-other=13`, and `support-hub=3`",
        "threshold 1000 lines, 30 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots",
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
            "threshold 1000 lines, 30 hotspots, 5 owner debt groups, 5 owner classes, and 0 unclassified hotspots"
        ),
        "hotspot inventory should carry the current 30-hotspot owner-budget gate, not only historical drift rows"
    );
    assert!(
        architecture_review.contains(
            "reports M1 gate status `migration-debt-present`, 30 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots"
        ),
        "architecture review should carry the current 30-hotspot large-file audit summary"
    );
    assert!(
        interface_doc.contains(
            "current audit output reports 30 hotspots above 1000 lines, M1 gate status `migration-debt-present`, 5 migration-debt owner groups, and zero unclassified hotspots"
        ),
        "interface convergence doc should carry the current 30-hotspot large-file audit summary"
    );
}

#[test]
fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts() {
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let ecs_doc = include_str!("../../../../../docs/zircon_runtime/scene/ecs.md");
    let interface_doc =
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let architecture_review =
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let audit_script = include_str!(
        "../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py"
    );
    let audit_source_inventory = include_str!(
        "../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
    );
    let audit_anchor_inventory = include_str!(
        "../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py"
    );
    let performance_guard = include_str!("../performance_hotspots.rs");
    let owner_budget_guard = include_str!("owner_budget.rs");
    let cargo_gate_guard = include_str!("../plan_status/cargo_gates/early.rs");

    for guard_anchor in [
        "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit",
        "runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
        "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
        "runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
        "runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
        "runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
        "runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
        "AnimationSceneFrameDiagnostics",
    ] {
        assert!(
            performance_guard.contains(guard_anchor)
                || owner_budget_guard.contains(guard_anchor)
                || cargo_gate_guard.contains(guard_anchor),
            "Runtime 07 guard anchor `{guard_anchor}` should stay visible to performance_hotpath_boundary"
        );
    }

    for source_inventory_anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 46",
        "EXPECTED_TEST_FILE_COUNT = 6",
    ] {
        assert!(
            audit_source_inventory.contains(source_inventory_anchor),
            "performance_hotpath_source_inventory should expose source/test audit anchor `{source_inventory_anchor}`"
        );
    }

    for anchor_inventory_anchor in [
        "ANIMATION_SCENE_ANCHORS",
        "MIRROR_DOCS_GUARD",
        "\"runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts\"",
    ] {
        assert!(
            audit_anchor_inventory.contains(anchor_inventory_anchor),
            "performance_hotpath_anchor_inventory should expose audit anchor `{anchor_inventory_anchor}`"
        );
    }

    for boundary_anchor in [
        "from runtime_structure_audits.performance_hotpath_source_inventory import",
        "from runtime_structure_audits.performance_hotpath_anchor_inventory import",
        "\"mirror_docs_guard_present\"",
    ] {
        assert!(
            audit_script.contains(boundary_anchor),
            "performance_hotpath_boundary should retain audit aggregation anchor `{boundary_anchor}`"
        );
    }

    let mirror_docs = [
        ("Runtime 07 plan", runtime_07_plan),
        ("runtime index", runtime_index),
        ("hotspot inventory doc", hotspot_doc),
        ("dynamic session doc", dynamic_session_doc),
        ("ECS doc", ecs_doc),
        ("runtime interface convergence doc", interface_doc),
        ("runtime architecture review", architecture_review),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for expected_anchor in [
            "performance_hotpath_boundary",
            "expected_source_file_count = 46",
            "expected_test_file_count = 6",
            "frame_span_anchor_count = 9",
            "query_counter_anchor_count = 32",
            "change_counter_anchor_count = 13",
            "extract_counter_anchor_count = 21",
            "asset_worker_anchor_count = 13",
            "animation_scene_anchor_count = 19",
            "profile_counter_hotspot_anchor_count = 8",
            "hotspot_guard_anchor_count = 32",
            "test_anchor_count = 29",
            "doc_anchor_count = 35",
            "cargo_gate_anchor_count = 5",
            "stale_hotspot_placeholder_present = false",
            "large_file_m1_gate_status = migration-debt-present",
            "large_file_hotspot_count = 30",
            "large_file_migration_debt_count = 5",
            "large_file_owner_class_count = 5",
            "large_file_unclassified_hotspot_count = 0",
            "missing_large_file_owner_classes = []",
            "missing_doc_anchors = []",
            "missing_cargo_gate_anchors = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(expected_anchor),
                "{doc_name} should mirror Runtime 07 performance-hotpath audit anchor `{expected_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed() {
    let root = include_str!("../../../core/framework/render/virtual_geometry_debug_snapshot.rs");
    let bvh = include_str!(
        "../../../core/framework/render/virtual_geometry_debug_snapshot/bvh_visualization.rs"
    );
    let cpu_reference = include_str!(
        "../../../core/framework/render/virtual_geometry_debug_snapshot/cpu_reference.rs"
    );
    let cull_input = include_str!(
        "../../../core/framework/render/virtual_geometry_debug_snapshot/cull_input.rs"
    );
    let execution =
        include_str!("../../../core/framework/render/virtual_geometry_debug_snapshot/execution.rs");
    let node_and_cluster = include_str!(
        "../../../core/framework/render/virtual_geometry_debug_snapshot/node_and_cluster_cull.rs"
    );
    let snapshot =
        include_str!("../../../core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs");
    let sources =
        include_str!("../../../core/framework/render/virtual_geometry_debug_snapshot/sources.rs");
    let module_doc = include_str!(
        "../../../../../docs/zircon_runtime/core/framework/render/virtual_geometry_debug_snapshot.md"
    );
    let hotspot_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");

    for root_anchor in [
        "mod bvh_visualization;",
        "mod cpu_reference;",
        "mod cull_input;",
        "mod execution;",
        "mod node_and_cluster_cull;",
        "mod snapshot;",
        "mod sources;",
        "pub use snapshot::RenderVirtualGeometryDebugSnapshot;",
    ] {
        assert!(
            root.contains(root_anchor),
            "virtual geometry debug snapshot root should stay structural with `{root_anchor}`"
        );
    }

    for root_forbidden in [
        "pub struct RenderVirtualGeometryDebugSnapshot {",
        "pub struct RenderVirtualGeometryCullInputSnapshot {",
        "pub struct RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {",
        "pub struct RenderVirtualGeometryCpuReferenceInstance {",
    ] {
        assert!(
            !root.contains(root_forbidden),
            "virtual geometry debug snapshot root should not regain owner declaration `{root_forbidden}`"
        );
    }

    for (source_name, source, owner_anchor) in [
        (
            "bvh_visualization.rs",
            bvh,
            "pub struct RenderVirtualGeometryBvhVisualizationInstance",
        ),
        (
            "cpu_reference.rs",
            cpu_reference,
            "pub struct RenderVirtualGeometryCpuReferenceInstance",
        ),
        (
            "cull_input.rs",
            cull_input,
            "pub struct RenderVirtualGeometryCullInputSnapshot",
        ),
        (
            "execution.rs",
            execution,
            "pub struct RenderVirtualGeometryVisBuffer64Entry",
        ),
        (
            "node_and_cluster_cull.rs",
            node_and_cluster,
            "pub struct RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot",
        ),
        (
            "snapshot.rs",
            snapshot,
            "pub struct RenderVirtualGeometryDebugSnapshot",
        ),
        (
            "sources.rs",
            sources,
            "pub enum RenderVirtualGeometryClusterSelectionInputSource",
        ),
    ] {
        assert!(
            source.contains(owner_anchor),
            "{source_name} should own `{owner_anchor}` after the folder split"
        );
    }

    assert!(
        node_and_cluster.contains("RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT"),
        "NodeAndClusterCull layout should consume the cull-input owner instead of duplicating cull words"
    );
    assert!(
        snapshot.contains("use super::node_and_cluster_cull::{"),
        "top-level debug snapshot should compose the NodeAndClusterCull owner through the folder boundary"
    );
    assert!(
        module_doc.contains(
            "virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,snapshot}.rs"
        ) && hotspot_doc
            .contains("virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred"),
        "Runtime 07 docs should record the virtual geometry debug snapshot owner split"
    );
}
