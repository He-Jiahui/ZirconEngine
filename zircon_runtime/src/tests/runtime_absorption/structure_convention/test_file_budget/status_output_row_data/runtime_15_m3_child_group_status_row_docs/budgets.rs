use super::*;

#[test]
fn runtime_15_m3_child_group_status_row_doc_child_budgets_stay_focused() {
    for (path, budget) in [
        (STATUS_DOCS_GUARD_PATH, 70),
        (STATUS_ROW_DOCS_GUARD_PATH, 70),
        (ROOT_PATHS_PATH, 120),
        (ROOT_STATUSES_PATH, 80),
        (ROOT_CHILD_ROWS_PATH, 120),
        (ROOT_SOURCE_BLOBS_PATH, 60),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        (LOCK_POISON_STATUS_ROWS_PATH, 400),
        (MODULE_CONVENTION_STATUS_ROWS_PATH, 400),
        (REVIEW_STATUS_SYNC_ROWS_PATH, 400),
        (STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH, 280),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the focused Runtime 15 M3 child-group row-doc guard budget of {budget} lines; got {line_count}"
        );
    }

    for (_, child_path, _) in STATUS_ROW_DOCS_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 170,
            "{child_path} should stay focused after M3 child-group status-row-doc folder-backed split; got {line_count} lines"
        );
    }

    for (path, budget) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors.rs",
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors/child_owner_status.rs",
            70,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors/m3_row_status.rs",
            110,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors/folder_backed_status.rs",
            80,
        ),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its status-mirror child budget of {budget} lines; got {line_count}"
        );
    }
}
