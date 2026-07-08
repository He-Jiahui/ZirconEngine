use super::*;

pub(super) fn assert_module_layout_child_summary_surrounding_owner_budgets_are_current() {
    for (path, source, budget) in [
        (
            "structure_convention/test_file_budget/row_data/module_layout.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout.rs",
            ),
            400,
        ),
        (
            "structure_convention/test_file_budget/row_data/module_layout_child_summary_status_docs.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status_docs.rs",
            ),
            400,
        ),
        (
            "structure_convention/test_file_budget/row_data/evidence_anchors.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/evidence_anchors.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/row_data/runtime_15_row_data.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_row_data.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups.rs",
            ),
            800,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the Runtime 15 test-file budget {budget}; got {line_count} lines"
        );
    }
}
