use super::*;

#[test]
fn runtime_15_module_layout_child_summary_guard_owner_budgets_are_child_owned() {
    let child_summary_parent = read_runtime_src(CHILD_SUMMARY_PARENT_PATH);
    assert!(
        child_summary_parent.lines().count() < 120,
        "module_layout_child_summaries.rs should stay below 120 lines as a route/shared-helper owner"
    );

    for (path, source) in child_summary_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 220,
            "{path} should stay below the focused Runtime 15 child-summary guard budget; got {line_count} lines"
        );
    }
    for (path, budget) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/foundation_review.rs",
            150,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/foundation_review/runtime_foundation_rows.rs",
            100,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/foundation_review/foundation_status_docs.rs",
            80,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/foundation_review/review_guard_rows.rs",
            130,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/milestone_groups.rs",
            150,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/milestone_groups/runtime_row_data.rs",
            90,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/milestone_groups/m3_child_groups.rs",
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/milestone_groups/status_doc_groups.rs",
            70,
        ),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its nested child-summary budget of {budget} lines; got {line_count}"
        );
    }

    for (path, source, budget) in [
        (
            "structure_convention/test_file_budget/status_output_row_data/module_layout.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout.rs",
            ),
            400,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs",
            ),
            400,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs.rs",
            ),
            800,
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
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
