use super::*;

#[test]
fn runtime_15_ui_asset_surface_index_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/asset_surface_index.rs");
    let dirty_targets = read_runtime_src("ui/tests/asset_surface_index/dirty_targets.rs");
    let node_resources = read_runtime_src("ui/tests/asset_surface_index/node_resources.rs");
    let surface_edges = read_runtime_src("ui/tests/asset_surface_index/surface_edges.rs");

    assert_contains_all(
        "UI asset surface-index parent mounts folder-backed children",
        &parent,
        &[
            "mod dirty_targets;",
            "mod node_resources;",
            "mod surface_edges;",
            "const TEMPLATE_WITH_RESOURCES",
            "fn dirty_test_surface(",
            "fn dirty_test_surface_with_nodes(",
            "fn resource_value(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/asset_surface_index.rs should only mount child test owners and shared fixtures"
    );
    for moved_test in [
        "surface_index_tracks_assets_and_replaces_stale_surface_edges",
        "surface_index_registers_node_resources_from_template_metadata",
        "hot_reload_plan_marks_target_surface_roots_dirty_and_reports_missing_surfaces",
        "template_plan_targets_surface_that_owns_compiled_asset",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI asset surface-index test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI asset surface-edge child owns surface dependency tests",
        &surface_edges,
        &[
            "fn surface_index_tracks_assets_and_replaces_stale_surface_edges",
            "fn surface_index_records_compiled_document_resource_dependencies",
            "fn hot_reload_plan_maps_template_theme_and_resource_targets_to_surfaces",
        ],
    );
    assert_contains_all(
        "UI asset node-resource child owns node dependency tests",
        &node_resources,
        &[
            "fn surface_index_tracks_node_asset_edges_and_replaces_stale_node_edges",
            "fn surface_index_registers_node_resources_from_template_metadata",
            "fn surface_index_tree_resource_registration_removes_stale_node_edges",
            "fn hot_reload_plan_maps_resource_targets_to_precise_nodes_when_registered",
        ],
    );
    assert_contains_all(
        "UI asset dirty-target child owns hot reload dirty application tests",
        &dirty_targets,
        &[
            "fn hot_reload_plan_marks_target_surface_roots_dirty_and_reports_missing_surfaces",
            "fn hot_reload_plan_marks_precise_resource_nodes_and_reports_missing_nodes",
            "fn mixed_surface_and_node_targets_fall_back_to_root_dirty",
            "fn template_rebuild_still_uses_surface_level_dirty_even_when_node_edges_exist",
            "fn template_plan_targets_surface_that_owns_compiled_asset",
        ],
    );

    let child_test_total = [
        dirty_targets.as_str(),
        node_resources.as_str(),
        surface_edges.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 12,
        "UI asset surface-index children should preserve all 12 parent tests"
    );

    for (path, source) in [
        ("ui/tests/asset_surface_index.rs", parent.as_str()),
        (
            "ui/tests/asset_surface_index/dirty_targets.rs",
            dirty_targets.as_str(),
        ),
        (
            "ui/tests/asset_surface_index/node_resources.rs",
            node_resources.as_str(),
        ),
        (
            "ui/tests/asset_surface_index/surface_edges.rs",
            surface_edges.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = ui_tests_second_status_row_source();
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI asset surface index test folder split",
                "runtime_15_ui_asset_surface_index_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/asset_surface_index.rs",
                "ui/tests/asset_surface_index/surface_edges.rs",
                "ui/tests/asset_surface_index/dirty_targets.rs",
                "runtime_15_ui_asset_surface_index_tests_are_folder_backed",
            ],
        );
    }
}
