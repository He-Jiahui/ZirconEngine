use super::*;

#[test]
fn runtime_15_module_layout_child_summary_foundation_status_docs_are_child_owned() {
    let runtime_15_foundation_row_data_status_docs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs.rs",
    );
    let runtime_15_foundation_row_data_status_doc_delegation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/delegation.rs",
    );
    let runtime_15_foundation_row_data_status_doc_status_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/status_maps.rs",
    );
    let runtime_15_foundation_row_data_status_doc_doc_mirrors = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/doc_mirrors.rs",
    );
    let runtime_15_foundation_row_data_status_doc_row_count = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count.rs",
    );
    let runtime_15_foundation_row_data_status_doc_children = [
        runtime_15_foundation_row_data_status_doc_delegation.as_str(),
        runtime_15_foundation_row_data_status_doc_status_maps.as_str(),
        runtime_15_foundation_row_data_status_doc_doc_mirrors.as_str(),
        runtime_15_foundation_row_data_status_doc_row_count.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 foundation row-data status-doc child owns status/doc anchors",
        &runtime_15_foundation_row_data_status_docs,
        &[
            "Runtime 15 M3 foundation row-data status-doc guard child-owner split",
            "runtime_15_foundation_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 foundation row-data status-doc guard folder-backed split",
            "runtime_15_foundation_row_data_status_docs_folder_backed_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 foundation row-data status-doc folder owns status/doc anchors",
        &runtime_15_foundation_row_data_status_doc_children,
        &["fn runtime_15_status_output_foundation_row_data_status_docs_are_child_owner"],
    );
}
