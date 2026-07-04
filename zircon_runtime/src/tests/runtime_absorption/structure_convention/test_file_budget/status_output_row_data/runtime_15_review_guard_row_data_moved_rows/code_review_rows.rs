use super::*;

#[path = "code_review_rows/plugin_importer_rows.rs"]
mod plugin_importer_rows;
#[path = "code_review_rows/review_guard_rows.rs"]
mod review_guard_rows;
#[path = "code_review_rows/source_delegation.rs"]
mod source_delegation;
#[path = "code_review_rows/status_mirrors.rs"]
mod status_mirrors;
#[path = "code_review_rows/structure_guard_rows.rs"]
mod structure_guard_rows;
#[path = "code_review_rows/typed_error_structure_rows.rs"]
mod typed_error_structure_rows;

const CODE_REVIEW_ROWS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs";
const CODE_REVIEW_ROWS_SOURCE_DELEGATION_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/source_delegation.rs";
const CODE_REVIEW_ROWS_REVIEW_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/review_guard_rows.rs";
const CODE_REVIEW_ROWS_STRUCTURE_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/structure_guard_rows.rs";
const CODE_REVIEW_ROWS_TYPED_ERROR_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/typed_error_structure_rows.rs";
const CODE_REVIEW_ROWS_PLUGIN_IMPORTER_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/plugin_importer_rows.rs";
const CODE_REVIEW_ROWS_STATUS_MIRRORS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/status_mirrors.rs";

const CODE_REVIEW_ROW_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "source_delegation",
        CODE_REVIEW_ROWS_SOURCE_DELEGATION_PATH,
        "assert_moved_code_review_row_sources_are_delegated",
    ),
    (
        "review_guard_rows",
        CODE_REVIEW_ROWS_REVIEW_GUARD_PATH,
        "assert_moved_review_guard_rows_are_child_owned",
    ),
    (
        "structure_guard_rows",
        CODE_REVIEW_ROWS_STRUCTURE_GUARD_PATH,
        "assert_moved_structure_guard_rows_are_child_owned",
    ),
    (
        "typed_error_structure_rows",
        CODE_REVIEW_ROWS_TYPED_ERROR_PATH,
        "assert_moved_typed_error_structure_rows_are_child_owned",
    ),
    (
        "plugin_importer_rows",
        CODE_REVIEW_ROWS_PLUGIN_IMPORTER_PATH,
        "assert_moved_plugin_importer_rows_are_child_owned",
    ),
    (
        "status_mirrors",
        CODE_REVIEW_ROWS_STATUS_MIRRORS_PATH,
        "runtime_15_review_guard_moved_row_code_review_rows_child_split_status_is_current",
    ),
];

#[test]
fn runtime_15_review_guard_moved_row_code_review_rows_are_child_owned() {
    let parent = read_runtime_src(CODE_REVIEW_ROWS_GUARD_PATH);
    for (module_name, path, guard_name) in CODE_REVIEW_ROW_CHILDREN {
        let module_mount = format!("#[path = \"code_review_rows/{module_name}.rs\"]");
        assert_contains_all(
            "review-guard moved-row code-review rows route mounts child",
            &parent,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child = read_runtime_src(path);
        assert_contains_all(path, &child, &[*guard_name]);
    }

    source_delegation::assert_moved_code_review_row_sources_are_delegated();
    review_guard_rows::assert_moved_review_guard_rows_are_child_owned();
    structure_guard_rows::assert_moved_structure_guard_rows_are_child_owned();
    typed_error_structure_rows::assert_moved_typed_error_structure_rows_are_child_owned();
    plugin_importer_rows::assert_moved_plugin_importer_rows_are_child_owned();
}

fn code_review_rows_child_source_blob() -> String {
    CODE_REVIEW_ROW_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}
