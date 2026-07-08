use super::*;

const MOVED_SUMMARY_LABELS: &[&str] = &[
    "evidence anchor child owns variable evidence guard",
    "Runtime 15 row-data child owns Runtime 15 parent split guard",
    "Runtime 15 foundation row-data child owns foundation split guard",
    "Runtime 15 foundation row-data status-doc child owns status/doc anchors",
    "Runtime 15 review-guard row-data child owns review-guard split guard",
    "Runtime 15 review-guard row-data moved-row child owns moved-row assertions",
    "Runtime 15 review-guard row-data status-doc child owns status/doc anchors",
    "Runtime 15 M4 row-data child owns M4 split guard",
    "Runtime 15 M2 row-data child owns M2 split guard",
    "Runtime 15 M3 row-data child owns M3 split guard",
    "Runtime 15 M3 child-groups guard owns M3 child split guard",
    "Runtime 15 M3 child-group moved-row child owns moved-row assertions",
    "Runtime 15 M3 child-group status-doc child owns status/doc anchors",
    "Runtime 15 M3 child-group status-row-doc child owns row status/doc anchors",
];

#[test]
fn runtime_15_status_output_row_data_module_layout_child_summaries_are_child_owner() {
    let status_output_row_data_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs",
    );
    let module_layout_guard = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout.rs",
    );
    let child_summary_parent = read_runtime_src(CHILD_SUMMARY_PARENT_PATH);
    let child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = child_summary_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts module-layout child-summary children",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/module_layout_child_summaries.rs\"]",
            "mod module_layout_child_summaries;",
            "#[path = \"row_data/module_layout_child_summary_status_docs.rs\"]",
            "mod module_layout_child_summary_status_docs;",
        ],
    );
    assert_contains_all(
        "module-layout child-summary guard routes focused child owners",
        &child_summary_parent,
        &[
            "mod delegation;",
            "mod foundation_review;",
            "mod milestone_groups;",
            "mod owner_budgets;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_paths;",
            "mod root_source_blobs;",
            "mod root_statuses;",
        ],
    );
    assert_contains_all(
        "module-layout child-summary guard records status anchors",
        &status_inventory,
        CHILD_SUMMARY_STATUS_ANCHORS,
    );

    for moved_summary in MOVED_SUMMARY_LABELS {
        assert!(
            !module_layout_guard.contains(moved_summary),
            "module_layout.rs should delegate child-summary assertion {moved_summary}"
        );
        assert!(
            child_sources.contains(moved_summary),
            "module_layout_child_summaries folder should own child-summary assertion {moved_summary}"
        );
    }

    for (module_name, child_path, guard_name) in CHILD_SUMMARY_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert!(
            child_summary_parent.contains(&module_mount),
            "module_layout_child_summaries.rs should mount child module `{module_mount}`"
        );
        assert!(
            child_inventory.contains(child_path),
            "module_layout_child_summaries child inventory should list {child_path}"
        );

        let child = read_runtime_src(child_path);
        assert_contains_all(child_path, &child, &["use super::*;", guard_name]);
        assert!(
            child.lines().count() < 220,
            "module-layout child-summary owner `{child_path}` should stay below 220 lines"
        );
    }

    assert!(
        !child_summary_parent.contains("#[test]"),
        "module_layout_child_summaries.rs should route children and keep test bodies in folder-backed child owners"
    );
    assert!(
        child_summary_parent.lines().count() < 120,
        "module_layout_child_summaries.rs should remain a small route/shared-helper owner"
    );
}
