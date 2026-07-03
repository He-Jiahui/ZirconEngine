use super::*;

#[test]
fn runtime_15_module_layout_child_summary_status_docs_guard_children_stay_focused() {
    for (label, path, budget) in CHILD_SUMMARY_STATUS_DOC_OWNER_BUDGETS {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below the Runtime 15 module-layout child-summary status-doc owner budget {budget}; got {line_count} lines"
        );
    }
    for (_, path, _) in CHILD_SUMMARY_STATUS_DOC_CHILDREN {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < 220,
            "{path} should stay below the focused Runtime 15 module-layout child-summary status-doc child budget; got {line_count} lines"
        );
    }
    for (child_path, budget) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/status_mirrors.rs",
            130,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/status_mirrors/historical_status.rs",
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/status_mirrors/folder_backed_status.rs",
            90,
        ),
    ] {
        let line_count = read_runtime_src(child_path).lines().count();
        assert!(
            line_count < budget,
            "{child_path} should stay below its child-summary status-mirror budget of {budget} lines; got {line_count}"
        );
    }
}
