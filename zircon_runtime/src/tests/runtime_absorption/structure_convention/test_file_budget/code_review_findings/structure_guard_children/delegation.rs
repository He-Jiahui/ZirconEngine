use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_structure_guard_children_are_mounted() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PARENT);
    let structure_guard = read_runtime_src(STRUCTURE_GUARD_CHILD_OWNER);
    let child_inventory = read_runtime_src(STRUCTURE_GUARD_ROOT_CHILD_ROWS_CHILD);
    let status_inventory = read_runtime_src(STRUCTURE_GUARD_ROOT_STATUSES_CHILD);
    let child_sources = structure_guard_child_source_blob();

    assert_contains_all(
        "code review findings structure guard parent mounts child owners",
        &parent,
        &[
            "#[path = \"code_review_findings/f8_child_owners.rs\"]",
            "mod f8_child_owners;",
            "#[path = \"code_review_findings/folder_backed_summary.rs\"]",
            "mod folder_backed_summary;",
            "#[path = \"code_review_findings/late_api_cleanup_child_owners.rs\"]",
            "mod late_api_cleanup_child_owners;",
            "#[path = \"code_review_findings/p0_child_owners.rs\"]",
            "mod p0_child_owners;",
            "#[path = \"code_review_findings/p0_native_fixture_leaf_owners.rs\"]",
            "mod p0_native_fixture_leaf_owners;",
            "#[path = \"code_review_findings/plugin_importer_dx_child_owners.rs\"]",
            "mod plugin_importer_dx_child_owners;",
            "#[path = \"code_review_findings/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"code_review_findings/structure_guard_children.rs\"]",
            "mod structure_guard_children;",
            "#[path = \"code_review_findings/typed_error_child_owners.rs\"]",
            "mod typed_error_child_owners;",
            "fn runtime_15_code_review_findings_tests_are_folder_backed",
            "folder_backed_summary::assert_code_review_findings_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "code review findings structure guard parent mounts folder-backed children",
        &structure_guard,
        &[
            "#[path = \"structure_guard_children/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"structure_guard_children/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure_guard_children/plugin_importer.rs\"]",
            "mod plugin_importer;",
            "#[path = \"structure_guard_children/review_guard_groups.rs\"]",
            "mod review_guard_groups;",
            "#[path = \"structure_guard_children/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"structure_guard_children/root_inventory.rs\"]",
            "mod root_inventory;",
            "#[path = \"structure_guard_children/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"structure_guard_children/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"structure_guard_children/root_statuses.rs\"]",
            "mod root_statuses;",
            "#[path = \"structure_guard_children/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"structure_guard_children/folder_backed_summary.rs\"]",
            "mod folder_backed_summary;",
            "#[path = \"structure_guard_children/typed_error.rs\"]",
            "mod typed_error;",
        ],
    );
    assert_contains_all(
        "code review findings structure guard root statuses keep folder-backed status",
        &status_inventory,
        &[
            STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_NAME,
            STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    for child_owned_test in [
        "fn runtime_15_f8_api_convergence_review_guards_are_child_owners",
        "fn runtime_15_code_review_findings_folder_backed_summary_is_child_owner",
        "fn runtime_15_late_api_cleanup_review_guards_are_child_owners",
        "fn runtime_15_p0_robustness_review_guards_are_child_owners",
        "fn runtime_15_p0_native_fixture_review_guards_are_leaf_owners",
        "fn runtime_15_code_review_findings_plugin_importer_dx_structure_guard_is_child_owner",
        "fn runtime_15_code_review_findings_status_docs_are_child_owner",
        "fn runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner",
        "fn runtime_15_code_review_findings_structure_guard_folder_backed_summary_is_child_owner",
        "fn runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner",
        "F8 structure child owner keeps F8 review guard ownership checks",
        "plugin-importer DX structure child owner keeps plugin DX review guard ownership checks",
        "code review findings status-doc structure child owner keeps status/document checks",
    ] {
        assert!(
            !structure_guard.contains(child_owned_test),
            "child-owned structure guard `{child_owned_test}` should not return to {STRUCTURE_GUARD_CHILD_OWNER}"
        );
    }
    for (_, child_path, guard_name) in STRUCTURE_GUARD_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "structure guard root child inventory should list child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "structure guard child {child_path} should define {guard_name}"
        );
    }

    review_guard_groups::assert_review_guard_group_children_are_mounted();
    plugin_importer::assert_plugin_importer_dx_children_are_mounted();
    status_docs::assert_code_review_status_doc_children_are_mounted();
    assert_nested_structure_children_are_mounted();
    budgets::assert_structure_guard_children_line_budgets();
}
