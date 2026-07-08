use super::*;

pub(super) fn assert_module_layout_child_summary_nested_budgets_are_current() {
    for (path, budget) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review.rs",
            150,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/runtime_foundation_rows.rs",
            100,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/foundation_status_docs.rs",
            80,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/review_guard_rows.rs",
            130,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups.rs",
            150,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/runtime_row_data.rs",
            90,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/m3_child_groups.rs",
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/status_doc_groups.rs",
            70,
        ),
        (ROOT_PATHS_PATH, 60),
        (ROOT_STATUSES_PATH, 40),
        (ROOT_CHILD_ROWS_PATH, 80),
        (ROOT_SOURCE_BLOBS_PATH, 40),
        (ROOT_INVENTORY_GUARD_PATH, 100),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its nested child-summary budget of {budget} lines; got {line_count}"
        );
    }
}
