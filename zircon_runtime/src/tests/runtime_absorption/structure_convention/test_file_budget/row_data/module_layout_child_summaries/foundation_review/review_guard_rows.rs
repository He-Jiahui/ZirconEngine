use super::*;

#[test]
fn runtime_15_module_layout_child_summary_review_guard_rows_are_child_owned() {
    let runtime_15_review_guard_row_data_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs",
    );
    let runtime_15_review_guard_row_data_delegation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation.rs",
    );
    let runtime_15_review_guard_row_data_delegation_route_mounts = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/route_mounts.rs",
    );
    let runtime_15_review_guard_row_data_root_statuses = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_statuses.rs",
    );
    let runtime_15_review_guard_row_data_sources = [
        runtime_15_review_guard_row_data_parent.as_str(),
        runtime_15_review_guard_row_data_delegation.as_str(),
        runtime_15_review_guard_row_data_delegation_route_mounts.as_str(),
        runtime_15_review_guard_row_data_root_statuses.as_str(),
    ]
    .join("\n");
    let runtime_15_review_guard_row_data_moved_rows = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows.rs",
    );
    let runtime_15_review_guard_row_data_moved_row_delegation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/delegation.rs",
    );
    let runtime_15_review_guard_row_data_moved_row_code_review = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs",
    );
    let runtime_15_review_guard_row_data_moved_row_typed_error = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs",
    );
    let runtime_15_review_guard_row_data_moved_row_status_mirrors = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs",
    );
    let runtime_15_review_guard_row_data_moved_row_root_statuses = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/root_statuses.rs",
    );
    let runtime_15_review_guard_row_data_moved_row_children = [
        runtime_15_review_guard_row_data_moved_row_delegation.as_str(),
        runtime_15_review_guard_row_data_moved_row_code_review.as_str(),
        runtime_15_review_guard_row_data_moved_row_typed_error.as_str(),
        runtime_15_review_guard_row_data_moved_row_status_mirrors.as_str(),
        runtime_15_review_guard_row_data_moved_row_root_statuses.as_str(),
    ]
    .join("\n");
    let runtime_15_review_guard_row_data_moved_row_sources = [
        runtime_15_review_guard_row_data_moved_rows.as_str(),
        runtime_15_review_guard_row_data_moved_row_children.as_str(),
    ]
    .join("\n");
    let runtime_15_review_guard_row_data_status_docs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status_docs.rs",
    );
    let runtime_15_review_guard_row_data_status_doc_root_statuses = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/root_statuses.rs",
    );
    let runtime_15_review_guard_row_data_status_doc_row_sources = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/row_sources.rs",
    );
    let runtime_15_review_guard_row_data_status_doc_sources = [
        runtime_15_review_guard_row_data_status_docs.as_str(),
        runtime_15_review_guard_row_data_status_doc_root_statuses.as_str(),
        runtime_15_review_guard_row_data_status_doc_row_sources.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 review-guard row-data child owns review-guard split guard",
        &runtime_15_review_guard_row_data_sources,
        &[
            "fn runtime_15_status_output_m3_review_guard_row_data_is_child_owner",
            "Runtime 15 M3 status output review-guard row-data guard child-owner split",
            "runtime_15_status_output_review_guard_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert!(!runtime_15_review_guard_row_data_parent.contains(
        concat!(
            "fn runtime_15_status_output_m3_review_guard_",
            "row_data_moved_rows_are_child_owner"
        )
    ), "runtime_15_review_guard_row_data.rs should delegate moved-row assertions to its child owner");
    assert_contains_all(
        "Runtime 15 review-guard row-data moved-row child owns moved-row assertions",
        &runtime_15_review_guard_row_data_moved_row_sources,
        &[
            "Runtime 15 M3 review-guard row-data moved-row guard child-owner split",
            "runtime_15_review_guard_row_data_moved_rows_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 review-guard moved-row guard folder-backed split",
            "runtime_15_review_guard_moved_row_guard_folder_backed_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 review-guard row-data moved-row folder owns moved-row assertions",
        &runtime_15_review_guard_row_data_moved_row_children,
        &[concat!(
            "fn runtime_15_status_output_m3_review_guard_",
            "row_data_moved_rows_are_child_owner"
        )],
    );
    assert_contains_all(
        "Runtime 15 review-guard row-data status-doc child owns status/doc anchors",
        &runtime_15_review_guard_row_data_status_doc_sources,
        &[
            "fn runtime_15_status_output_review_guard_row_data_status_docs_are_child_owner",
            "Runtime 15 M3 review-guard row-data status-doc guard child-owner split",
            "runtime_15_review_guard_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}
