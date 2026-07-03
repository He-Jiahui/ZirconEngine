use super::*;

#[test]
fn runtime_15_status_output_row_data_module_layout_child_summaries_stay_delegated() {
    let module_layout_guard = read_runtime_src(MODULE_LAYOUT_PARENT_PATH);
    let module_layout_child_summaries = read_runtime_src(MODULE_LAYOUT_CHILD_SUMMARIES_PATH);
    let module_layout_child_summary_delegation =
        read_runtime_src(MODULE_LAYOUT_CHILD_SUMMARY_DELEGATION_PATH);
    let module_layout_child_summary_foundation_review =
        read_runtime_src(MODULE_LAYOUT_CHILD_SUMMARY_FOUNDATION_REVIEW_PATH);
    let module_layout_child_summary_milestone_groups =
        read_runtime_src(MODULE_LAYOUT_CHILD_SUMMARY_MILESTONE_GROUPS_PATH);
    let module_layout_child_summary_owner_budgets =
        read_runtime_src(MODULE_LAYOUT_CHILD_SUMMARY_OWNER_BUDGETS_PATH);
    let module_layout_child_summary_status_docs_parent =
        read_runtime_src(MODULE_LAYOUT_CHILD_SUMMARY_STATUS_DOCS_PATH);
    let module_layout_child_summary_status_docs = format!(
        "{}\n{}",
        module_layout_child_summary_status_docs_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/status_mirrors.rs",
        )
    );
    let module_layout_status_docs_parent = read_runtime_src(MODULE_LAYOUT_STATUS_DOCS_PATH);
    let module_layout_status_docs = format!(
        "{}\n{}",
        module_layout_status_docs_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_status_docs/status_mirrors.rs",
        )
    );
    let module_layout_child_summary_children = [
        module_layout_child_summary_delegation.as_str(),
        module_layout_child_summary_foundation_review.as_str(),
        module_layout_child_summary_milestone_groups.as_str(),
        module_layout_child_summary_owner_budgets.as_str(),
    ]
    .join("\n");

    for moved_summary in [
        concat!("evidence anchor child owns ", "variable evidence guard"),
        concat!(
            "Runtime 15 foundation row-data child owns ",
            "foundation split guard"
        ),
        concat!("Runtime 15 M2 row-data child owns ", "M2 split guard"),
        concat!(
            "Runtime 15 M3 child-group moved-row child owns ",
            "moved-row assertions"
        ),
    ] {
        assert!(
            !module_layout_guard.contains(moved_summary),
            "module_layout.rs should delegate child-summary assertion {moved_summary}"
        );
        assert!(
            module_layout_child_summary_children.contains(moved_summary),
            "module_layout_child_summaries folder should own child-summary assertion {moved_summary}"
        );
    }
    assert_contains_all(
        "Runtime 15 module-layout child-summary guard is folder-backed",
        &module_layout_child_summaries,
        &[
            "mod delegation;",
            "mod foundation_review;",
            "mod milestone_groups;",
            "mod owner_budgets;",
            "Runtime 15 M3 module-layout child-summary guard folder-backed split",
            "runtime_15_module_layout_child_summary_guard_folder_backed_static_passed_cargo_deferred",
            "Runtime 15 M3 status output row-data module-layout child-summary guard child-owner split",
            "runtime_15_status_output_row_data_module_layout_child_summary_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 module-layout child-summary delegation guard owns child summaries",
        &module_layout_child_summary_delegation,
        &["fn runtime_15_status_output_row_data_module_layout_child_summaries_are_child_owner"],
    );
    assert_contains_all(
        "Runtime 15 module-layout child-summary status-doc child owns status/doc anchors",
        &module_layout_child_summary_status_docs,
        &[
            "fn runtime_15_module_layout_child_summary_status_docs_are_child_owner",
            "Runtime 15 M3 module-layout child-summary status-doc guard child-owner split",
            "runtime_15_module_layout_child_summary_status_docs_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 module-layout status-doc child owns status/doc anchors",
        &module_layout_status_docs,
        &["fn runtime_15_status_output_row_data_module_layout_status_docs_are_child_owner"],
    );
}
