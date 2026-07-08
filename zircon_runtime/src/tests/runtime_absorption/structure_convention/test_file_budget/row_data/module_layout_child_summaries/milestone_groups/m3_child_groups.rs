use super::*;

#[test]
fn runtime_15_module_layout_child_summary_m3_child_groups_are_child_owned() {
    let runtime_15_m3_child_groups_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups.rs",
    );
    let runtime_15_m3_child_groups = format!(
        "{}\n{}\n{}",
        runtime_15_m3_child_groups_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/exports.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/top_level.rs",
        )
    );
    let runtime_15_m3_child_group_moved_rows_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows.rs",
    );
    let runtime_15_m3_child_group_moved_rows = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        runtime_15_m3_child_group_moved_rows_parent,
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/delegation.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/lock_poison_rows.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/module_convention_rows.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/review_top_rows.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/status_mirrors.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/budgets.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/root_statuses.rs")
    );
    let runtime_15_m3_child_group_status_docs = format!(
        "{}\n{}\n{}\n{}",
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_docs.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/root_statuses.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/status_mirrors.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/mirrors/historical_status.rs")
    );
    let runtime_15_m3_child_group_status_row_docs = format!(
        "{}\n{}\n{}",
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/root_statuses.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/row_sources.rs")
    );

    assert_contains_all(
        "Runtime 15 M3 child-groups guard owns M3 child split guard",
        &runtime_15_m3_child_groups,
        &[
            "fn runtime_15_status_output_m3_row_data_child_owner_split",
            "top-level status rows include every Runtime 15 M3 child group",
        ],
    );
    assert!(
        !runtime_15_m3_child_groups
            .contains("fn runtime_15_status_output_m3_child_group_moved_rows_are_child_owner"),
        "runtime_15_m3_child_groups.rs should delegate moved-row checks to its child owner"
    );
    assert_contains_all(
        "Runtime 15 M3 child-group moved-row child owns moved-row assertions",
        &runtime_15_m3_child_group_moved_rows,
        &[
            "fn runtime_15_status_output_m3_child_group_moved_rows_are_child_owner",
            "Runtime 15 M3 child-group moved-row guard child-owner split",
            "runtime_15_m3_child_group_moved_row_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 child-group status-doc child owns status/doc anchors",
        &runtime_15_m3_child_group_status_docs,
        &[
            "fn runtime_15_status_output_m3_child_group_status_docs_are_child_owner",
            "Runtime 15 M3 child-groups status-doc guard child-owner split",
            "runtime_15_m3_child_groups_status_docs_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 child-group status-row-doc child owns row status/doc anchors",
        &runtime_15_m3_child_group_status_row_docs,
        &[
            "fn runtime_15_status_output_m3_child_group_status_row_docs_are_child_owner",
            "Runtime 15 M3 child-group status-row-doc guard child-owner split",
            "runtime_15_m3_child_group_status_row_docs_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}
