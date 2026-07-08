use super::*;

#[test]
fn runtime_15_module_layout_child_summary_runtime_foundation_rows_are_child_owned() {
    let evidence_anchors_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/evidence_anchors.rs",
    );
    let evidence_anchors = format!(
        "{}\n{}",
        evidence_anchors_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/evidence_anchors/variable_evidence.rs",
        )
    );
    let runtime_15_row_data = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_row_data.rs",
    );
    let runtime_15_row_data_row_ownership = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_row_data/row_ownership.rs",
    );
    let runtime_15_row_data_row_ownership_group_exports = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_row_data/row_ownership/group_exports.rs",
    );
    let runtime_15_row_data_sources = [
        runtime_15_row_data.as_str(),
        runtime_15_row_data_row_ownership.as_str(),
        runtime_15_row_data_row_ownership_group_exports.as_str(),
    ]
    .join("\n");
    let runtime_15_foundation_row_data = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data.rs",
    );
    let runtime_15_foundation_row_data_row_ownership = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data/row_ownership.rs",
    );
    let runtime_15_foundation_row_data_root_statuses = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data/root_statuses.rs",
    );
    let runtime_15_foundation_row_data_sources = [
        runtime_15_foundation_row_data.as_str(),
        runtime_15_foundation_row_data_row_ownership.as_str(),
        runtime_15_foundation_row_data_root_statuses.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "evidence anchor child owns variable evidence guard",
        &evidence_anchors,
        &[
            "fn runtime_15_expected_status_output_rows_accept_variable_evidence_anchors",
            "Runtime 15 M3 keeps multi-anchor evidence rows as slices",
        ],
    );
    assert_contains_all(
        "Runtime 15 row-data child owns Runtime 15 parent split guard",
        &runtime_15_row_data_sources,
        &[
            "fn runtime_15_status_output_runtime_15_row_data_is_child_owner",
            "status row data parent keeps only group aggregation",
        ],
    );
    for delegated_test in [
        "fn runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
        "fn runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
    ] {
        assert!(
            !runtime_15_row_data_sources.contains(delegated_test),
            "runtime_15_row_data.rs should delegate {delegated_test}"
        );
    }
    assert_contains_all(
        "Runtime 15 foundation row-data child owns foundation split guard",
        &runtime_15_foundation_row_data_sources,
        &[
            "fn runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
            "Runtime 15 M3 foundation row-data guard child-owner split",
            "runtime_15_foundation_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}
