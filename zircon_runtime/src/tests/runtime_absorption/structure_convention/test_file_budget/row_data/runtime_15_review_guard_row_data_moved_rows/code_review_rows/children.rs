pub(super) const CODE_REVIEW_ROWS_GUARD_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs";
pub(super) const CODE_REVIEW_ROWS_CHILDREN_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/children.rs";
pub(super) const CODE_REVIEW_ROWS_CHILD_OWNERSHIP_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/child_ownership.rs";
pub(super) const CODE_REVIEW_ROWS_SOURCE_DELEGATION_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/source_delegation.rs";
pub(super) const CODE_REVIEW_ROWS_REVIEW_GUARD_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/review_guard_rows.rs";
pub(super) const CODE_REVIEW_ROWS_STRUCTURE_GUARD_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/structure_guard_rows.rs";
pub(super) const CODE_REVIEW_ROWS_TYPED_ERROR_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/typed_error_structure_rows.rs";
pub(super) const CODE_REVIEW_ROWS_PLUGIN_IMPORTER_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/plugin_importer_rows.rs";
pub(super) const CODE_REVIEW_ROWS_STATUS_MIRRORS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/status_mirrors.rs";

pub(super) const CODE_REVIEW_ROW_CHILDREN: &[(&str, &str, &str)] = &[
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

pub(super) const CODE_REVIEW_ROWS_ROUTE_METADATA_CHILDREN: &[(&str, &str)] = &[
    ("children", CODE_REVIEW_ROWS_CHILDREN_PATH),
    ("child_ownership", CODE_REVIEW_ROWS_CHILD_OWNERSHIP_PATH),
    ("source_delegation", CODE_REVIEW_ROWS_SOURCE_DELEGATION_PATH),
    ("review_guard_rows", CODE_REVIEW_ROWS_REVIEW_GUARD_PATH),
    (
        "structure_guard_rows",
        CODE_REVIEW_ROWS_STRUCTURE_GUARD_PATH,
    ),
    (
        "typed_error_structure_rows",
        CODE_REVIEW_ROWS_TYPED_ERROR_PATH,
    ),
    (
        "plugin_importer_rows",
        CODE_REVIEW_ROWS_PLUGIN_IMPORTER_PATH,
    ),
    ("status_mirrors", CODE_REVIEW_ROWS_STATUS_MIRRORS_PATH),
];

pub(in super::super) fn code_review_rows_child_source_blob() -> String {
    CODE_REVIEW_ROW_CHILDREN
        .iter()
        .map(|(_, path, _)| super::read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}
