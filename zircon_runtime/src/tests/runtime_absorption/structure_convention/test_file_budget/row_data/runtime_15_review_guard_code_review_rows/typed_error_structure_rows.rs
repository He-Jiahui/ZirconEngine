use super::*;

#[path = "typed_error_structure_rows/budgets.rs"]
mod budgets;
#[path = "typed_error_structure_rows/delegation.rs"]
mod delegation;
#[path = "typed_error_structure_rows/folder_backed.rs"]
mod folder_backed;
#[path = "typed_error_structure_rows/row_groups.rs"]
mod row_groups;
#[path = "typed_error_structure_rows/status_doc_paths.rs"]
mod status_doc_paths;
#[path = "typed_error_structure_rows/status_mirrors.rs"]
mod status_mirrors;

const TYPED_ERROR_STRUCTURE_ROWS_STATUS_OUTPUT_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows.rs";
const TYPED_ERROR_STRUCTURE_ROW_GROUPS: &[(&str, &str)] = &[
    ("core_rows", "EXPECTED_STATUS_OUTPUT_SLICES"),
    (
        "status_doc_path_rows",
        "STATUS_DOC_PATHS_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_doc_delegation_rows",
        "STATUS_DOC_DELEGATION_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_doc_status_maps_rows",
        "STATUS_DOC_STATUS_MAPS_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_doc_status_mirrors_rows",
        "STATUS_DOC_STATUS_MIRRORS_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "structure_assertion_rows",
        "STRUCTURE_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "row_data_owner",
        "ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
];

#[test]
fn runtime_15_typed_error_structure_rows_guard_is_folder_backed() {
    folder_backed::assert_typed_error_structure_rows_guard_is_folder_backed();
    status_mirrors::assert_typed_error_structure_rows_guard_status_is_current();
}

fn typed_error_structure_rows_guard_child_source_blob() -> String {
    TYPED_ERROR_STRUCTURE_ROWS_STATUS_OUTPUT_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

const TYPED_ERROR_STRUCTURE_ROWS_STATUS_OUTPUT_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/budgets.rs",
        "runtime_15_typed_error_structure_rows_guard_children_line_budgets_are_current",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/delegation.rs",
        "runtime_15_typed_error_structure_rows_row_data_owner_is_child_backed",
    ),
    (
        "folder_backed",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/folder_backed.rs",
        "typed-error structure row-data guard route mounts folder-backed children",
    ),
    (
        "row_groups",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/row_groups.rs",
        "assert_typed_error_structure_row_groups_are_child_backed",
    ),
    (
        "status_doc_paths",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/status_doc_paths.rs",
        "assert_status_doc_paths_rows_are_child_backed",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/status_mirrors.rs",
        "assert_typed_error_structure_row_data_status_is_current",
    ),
];
