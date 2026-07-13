use super::*;

pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_assertions.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_ROW_DATA_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_ROW_DATA_STATUS_PATH: &str = "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/code_review_maps.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/code_review_maps.rs";
const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_ROOT: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_ROW_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/typed_error_guard_rows.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CORE_ROWS_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/core_rows.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTION_ROWS_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertion_rows.rs";

pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILDREN: &[(&str, &str)] = &[
    (
        "convergence_mounts",
        "CONVERGENCE_MOUNTS_ROOT_INVENTORY_CHILD_SPLIT",
    ),
    (
        "foundation",
        "STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_SPLIT",
    ),
    (
        "moved_guard_absence",
        "MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILD_SPLIT",
    ),
    (
        "native_plugin_loader",
        "NATIVE_PLUGIN_LOADER_ROUTES_CHILD_SPLIT",
    ),
];

pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/budgets.rs",
        "runtime_15_review_guard_typed_error_structure_assertions_guard_children_line_budgets_are_current",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/delegation.rs",
        "runtime_15_review_guard_typed_error_structure_assertions_row_data_is_folder_backed",
    ),
    (
        "folder_backed",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/folder_backed.rs",
        "assert_typed_error_structure_assertions_guard_is_folder_backed",
    ),
    (
        "paths",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/paths.rs",
        "typed_error_structure_assertions_guard_child_source_blob",
    ),
    (
        "row_routes",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/row_routes.rs",
        "assert_typed_error_structure_assertion_row_routes_are_child_backed",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/status_mirrors.rs",
        "assert_typed_error_structure_assertions_guard_status_is_current",
    ),
];

pub(super) fn typed_error_structure_assertions_guard_child_source_blob() -> String {
    TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn typed_error_structure_assertions_child_blob() -> String {
    TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILDREN
        .iter()
        .map(|(module_name, _)| {
            read_runtime_src(&typed_error_structure_assertions_child_path(module_name))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn typed_error_structure_assertions_child_path(module_name: &str) -> String {
    format!("{TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_ROOT}/{module_name}.rs")
}
