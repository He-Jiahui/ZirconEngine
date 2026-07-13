use super::*;

pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status_docs.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_ROW_DATA_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_ROW_DATA_STATUS_PATH: &str = "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_ROW_DATA_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/status_doc_maps.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_ROW_DATA_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/status_doc_maps.rs";
const TYPED_ERROR_STATUS_DOCS_ROW_DATA_CHILD_ROOT: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs";
pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_ROW_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/typed_error_guard_rows.rs";

pub(super) const TYPED_ERROR_STATUS_DOCS_ROW_DATA_CHILDREN: &[(&str, &str)] = &[
    (
        "delegation",
        "STATUS_DOC_DELEGATION_STATUS_CURRENT_SOURCES_CHILD_SPLIT",
    ),
    ("foundation", "DOC_MIRRORS_SOURCE_HELPER_CHILD_SPLIT"),
    (
        "paths",
        "STATUS_DOC_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_SOURCES_CHILD_SPLIT",
    ),
    (
        "status_maps",
        "STATUS_DOC_STATUS_MAPS_STATUS_CURRENT_SOURCES_CHILD_SPLIT",
    ),
    (
        "status_mirrors",
        "STATUS_DOC_STATUS_MIRRORS_STATUS_CURRENT_SOURCES_CHILD_SPLIT",
    ),
];

pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/budgets.rs",
        "runtime_15_review_guard_typed_error_status_docs_guard_children_line_budgets_are_current",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/delegation.rs",
        "runtime_15_review_guard_typed_error_status_docs_row_data_is_folder_backed",
    ),
    (
        "folder_backed",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/folder_backed.rs",
        "assert_typed_error_status_docs_guard_is_folder_backed",
    ),
    (
        "paths",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/paths.rs",
        "typed_error_status_docs_guard_child_source_blob",
    ),
    (
        "row_routes",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/row_routes.rs",
        "assert_typed_error_status_doc_row_routes_are_child_backed",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/status_mirrors.rs",
        "assert_typed_error_status_doc_guard_status_is_current",
    ),
];

pub(super) fn typed_error_status_docs_guard_child_source_blob() -> String {
    TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn typed_error_status_docs_row_data_child_blob() -> String {
    TYPED_ERROR_STATUS_DOCS_ROW_DATA_CHILDREN
        .iter()
        .map(|(module_name, _)| typed_error_status_docs_row_data_child_tree_blob(module_name))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn typed_error_status_docs_row_data_child_tree_blob(module_name: &str) -> String {
    typed_error_status_docs_row_data_child_tree_paths(module_name)
        .into_iter()
        .map(|path| read_runtime_src(&path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn typed_error_status_docs_row_data_child_tree_paths(module_name: &str) -> Vec<String> {
    let mut paths = vec![typed_error_status_docs_row_data_child_path(module_name)];
    let nested = match module_name {
        "delegation" => &["core", "sources", "split_layout"][..],
        "paths" => &["core", "status_current", "child_inventory"][..],
        "status_maps" => &["core", "sources", "split_layout"][..],
        "status_mirrors" => &["core", "sources", "split_layout"][..],
        _ => &[][..],
    };
    paths.extend(nested.iter().map(|child| {
        format!("{TYPED_ERROR_STATUS_DOCS_ROW_DATA_CHILD_ROOT}/{module_name}/{child}.rs")
    }));
    paths
}

pub(super) fn typed_error_status_docs_row_data_child_path(module_name: &str) -> String {
    format!("{TYPED_ERROR_STATUS_DOCS_ROW_DATA_CHILD_ROOT}/{module_name}.rs")
}
