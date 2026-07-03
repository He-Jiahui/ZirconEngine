use super::*;

#[test]
fn runtime_15_module_layout_status_docs_guard_children_stay_focused() {
    for (label, path, budget) in MODULE_LAYOUT_STATUS_DOC_OWNER_BUDGETS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its module-layout status-doc owner budget of {budget} lines; got {line_count}"
        );
    }
    for (_, child_path, _) in MODULE_LAYOUT_STATUS_DOC_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{child_path} should stay focused after module-layout status-doc folder-backed split; got {line_count}"
        );
    }
    for (child_path, budget) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_status_docs/status_mirrors.rs",
            130,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_status_docs/status_mirrors/historical_status.rs",
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_status_docs/status_mirrors/folder_backed_status.rs",
            80,
        ),
    ] {
        let line_count = read_runtime_src(child_path).lines().count();
        assert!(
            line_count < budget,
            "{child_path} should stay below its status-mirror child budget of {budget} lines; got {line_count}"
        );
    }
}
