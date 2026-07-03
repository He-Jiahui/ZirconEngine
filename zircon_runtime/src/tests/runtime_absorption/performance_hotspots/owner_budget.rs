#[path = "owner_budget/large_file_gate.rs"]
mod large_file_gate;
#[path = "owner_budget/mirror_docs.rs"]
mod mirror_docs;
#[path = "owner_budget/virtual_geometry_debug_snapshot.rs"]
mod virtual_geometry_debug_snapshot;

#[test]
fn runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed() {
    fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
        for anchor in anchors {
            assert!(
                source.contains(anchor),
                "{label} should retain Runtime 15 performance-hotspot guard anchor `{anchor}`"
            );
        }
    }

    let parent = include_str!("../performance_hotspots.rs");
    let artifact_render_diagnostics = include_str!("artifact_render_diagnostics_splits.rs");
    let hotspot_inventory = include_str!("hotspot_inventory.rs");
    let owner_budget = include_str!("owner_budget.rs");
    let owner_budget_large_file_gate = include_str!("owner_budget/large_file_gate.rs");
    let owner_budget_mirror_docs = include_str!("owner_budget/mirror_docs.rs");
    let owner_budget_virtual_geometry_debug_snapshot =
        include_str!("owner_budget/virtual_geometry_debug_snapshot.rs");
    let scene_project_splits = include_str!("scene_project_splits.rs");
    let submit_context = include_str!("submit_context.rs");
    let submit_error_paths = include_str!("submit_error_paths.rs");
    let source_inventory = include_str!(
        "../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
    );
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let hotspot_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let status_rows = include_str!(
        "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs"
    );
    let status_slice = include_str!(
        "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs"
    );
    let date_slice = include_str!(
        "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs"
    );

    assert_contains_all(
        "performance_hotspots parent",
        parent,
        &[
            "mod artifact_render_diagnostics_splits;",
            "mod hotspot_inventory;",
            "mod owner_budget;",
            "mod scene_project_splits;",
            "mod submit_context;",
            "mod submit_error_paths;",
        ],
    );

    for moved_guard in [
        "fn runtime_07_submit_context_shares_large_extract_payloads",
        "fn runtime_07_submit_paths_return_errors_for_checked_viewport_records",
        "fn runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "fn runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
        "fn runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
        "fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
        "fn runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
        "fn runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "performance_hotspots.rs should mount child owners instead of defining `{moved_guard}`"
        );
    }

    assert_contains_all(
        "submit context child",
        submit_context,
        &["fn runtime_07_submit_context_shares_large_extract_payloads"],
    );
    assert_contains_all(
        "submit error paths child",
        submit_error_paths,
        &["fn runtime_07_submit_paths_return_errors_for_checked_viewport_records"],
    );
    assert_contains_all(
        "hotspot inventory child",
        hotspot_inventory,
        &["fn runtime_07_hotspot_inventory_requires_counted_evidence_before_m2"],
    );
    assert_contains_all(
        "scene/project split child",
        scene_project_splits,
        &[
            "fn runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
            "fn runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
            "fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
        ],
    );
    assert_contains_all(
        "artifact/render diagnostics split child",
        artifact_render_diagnostics,
        &[
            "fn runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
            "fn runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
        ],
    );
    assert_contains_all(
        "owner-budget parent",
        owner_budget,
        &[
            "#[path = \"owner_budget/large_file_gate.rs\"]",
            "#[path = \"owner_budget/mirror_docs.rs\"]",
            "#[path = \"owner_budget/virtual_geometry_debug_snapshot.rs\"]",
            "fn runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
        ],
    );

    for moved_owner_budget_guard_name in [
        "runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit",
        "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        "runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
    ] {
        let moved_owner_budget_guard = format!("fn {moved_owner_budget_guard_name}");
        assert!(
            !owner_budget.contains(&moved_owner_budget_guard),
            "performance_hotspots/owner_budget.rs should mount child owners instead of defining `{moved_owner_budget_guard}`"
        );
    }

    let mirror_docs_guard = format!(
        "{}{}",
        "fn ", "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts"
    );
    assert!(
        owner_budget_mirror_docs.contains(&mirror_docs_guard),
        "owner-budget mirror docs child should retain Runtime 07 audit mirror guard"
    );
    assert_contains_all(
        "owner-budget mirror docs child",
        owner_budget_mirror_docs,
        &[
            "EXPECTED_TEST_FILE_COUNT = 14",
            "owner_budget/large_file_gate.rs",
            "owner_budget/mirror_docs.rs",
            "owner_budget/virtual_geometry_debug_snapshot.rs",
        ],
    );

    for (path, source) in [
        ("tests/runtime_absorption/performance_hotspots.rs", parent),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits.rs",
            artifact_render_diagnostics,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            hotspot_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget.rs",
            owner_budget,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs",
            owner_budget_large_file_gate,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
            owner_budget_mirror_docs,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/virtual_geometry_debug_snapshot.rs",
            owner_budget_virtual_geometry_debug_snapshot,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits.rs",
            scene_project_splits,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_context.rs",
            submit_context,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/submit_error_paths.rs",
            submit_error_paths,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "performance hotpath source inventory",
        source_inventory,
        &[
            "EXPECTED_TEST_FILE_COUNT = 14",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("Runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("status-output row data", status_rows),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
                "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
                "tests/runtime_absorption/performance_hotspots.rs",
                "tests/runtime_absorption/performance_hotspots/submit_context.rs",
                "runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 07 plan", runtime_07_plan),
        ("Runtime index", runtime_index),
        ("hotspot inventory doc", hotspot_doc),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "expected_test_file_count = 14",
                "performance_hotspots/owner_budget/{large_file_gate,mirror_docs,virtual_geometry_debug_snapshot}.rs",
            ],
        );
    }

    assert_contains_all(
        "status-output slices",
        &format!("{status_slice}\n{date_slice}"),
        &[
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
            "2026-06-23",
        ],
    );
}
