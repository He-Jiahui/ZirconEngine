#[test]
fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts() {
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");
    let interface_doc =
        include_str!("../../../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let architecture_review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let audit_script = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py"
    );
    let audit_source_inventory = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
    );
    let audit_anchor_inventory = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py"
    );
    let performance_guard = include_str!("../../performance_hotspots.rs");
    let artifact_render_diagnostics_guard =
        include_str!("../artifact_render_diagnostics_splits.rs");
    let hotspot_inventory_guard = include_str!("../hotspot_inventory.rs");
    let owner_budget_guard = include_str!("../owner_budget.rs");
    let owner_budget_large_file_guard = include_str!("large_file_gate.rs");
    let owner_budget_mirror_docs_guard = include_str!("mirror_docs.rs");
    let owner_budget_virtual_geometry_debug_snapshot_guard =
        include_str!("virtual_geometry_debug_snapshot.rs");
    let scene_project_splits_guard = include_str!("../scene_project_splits.rs");
    let submit_context_guard = include_str!("../submit_context.rs");
    let submit_error_paths_guard = include_str!("../submit_error_paths.rs");
    let cargo_gate_guard = include_str!("../../plan_status/cargo_gates/early.rs");
    let performance_guard_sources = [
        performance_guard,
        artifact_render_diagnostics_guard,
        hotspot_inventory_guard,
        owner_budget_guard,
        owner_budget_large_file_guard,
        owner_budget_mirror_docs_guard,
        owner_budget_virtual_geometry_debug_snapshot_guard,
        scene_project_splits_guard,
        submit_context_guard,
        submit_error_paths_guard,
        cargo_gate_guard,
    ];

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
            performance_guard_sources
                .iter()
                .any(|source| source.contains(guard_anchor)),
            "Runtime 07 guard anchor `{guard_anchor}` should stay visible to performance_hotpath_boundary"
        );
    }

    for source_inventory_anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 46",
        "EXPECTED_TEST_FILE_COUNT = 14",
        "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs",
        "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
        "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/virtual_geometry_debug_snapshot.rs",
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
            "expected_test_file_count = 14",
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
            "large_file_m1_gate_status = classified-and-clear",
            "large_file_hotspot_count = 0",
            "large_file_migration_debt_count = 0",
            "large_file_owner_class_count = 0",
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
