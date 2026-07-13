use super::*;

#[test]
fn runtime_15_module_layout_child_summary_status_doc_groups_are_child_owned() {
    let module_layout_status_docs_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status_docs.rs",
    );
    let module_layout_status_docs = format!(
        "{}\n{}\n{}",
        module_layout_status_docs_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/status_mirrors.rs"
        ),
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/mirrors/historical_status.rs"
        )
    );
    let module_layout_child_summary_status_docs_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status_docs.rs",
    );
    let module_layout_child_summary_status_docs = format!(
        "{}\n{}\n{}\n{}",
        module_layout_child_summary_status_docs_parent,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status/root_statuses.rs"
        ),
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status/status_mirrors.rs"
        ),
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summary_status/mirrors/historical_status.rs"
        )
    );

    assert_contains_all(
        "Runtime 15 module-layout status-doc child owns status/doc anchors",
        &module_layout_status_docs,
        &["fn runtime_15_status_output_row_data_module_layout_status_docs_are_child_owner"],
    );
    assert_contains_all(
        "Runtime 15 module-layout child-summary status-doc child owns status/doc anchors",
        &module_layout_child_summary_status_docs,
        &[
            "fn runtime_15_module_layout_child_summary_status_docs_are_child_owner",
            "Runtime 15 M3 module-layout child-summary status-doc guard child-owner split",
            "runtime_15_module_layout_child_summary_status_docs_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}
