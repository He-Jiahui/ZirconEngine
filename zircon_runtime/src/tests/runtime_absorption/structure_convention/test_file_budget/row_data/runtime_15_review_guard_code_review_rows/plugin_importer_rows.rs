use super::*;

#[path = "plugin_importer_rows/budgets.rs"]
mod budgets;
#[path = "plugin_importer_rows/child_split_status.rs"]
mod child_split_status;
#[path = "plugin_importer_rows/delegation.rs"]
mod delegation;
#[path = "plugin_importer_rows/row_children.rs"]
mod row_children;
#[path = "plugin_importer_rows/row_data_status.rs"]
mod row_data_status;
#[path = "plugin_importer_rows/status_mirrors.rs"]
mod status_mirrors;

const PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows.rs";
const PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/budgets.rs",
        "runtime_15_plugin_importer_rows_status_output_guard_children_line_budgets_are_current",
    ),
    (
        "child_split_status",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/child_split_status.rs",
        "runtime_15_plugin_importer_status_output_guard_folder_backed_status_is_current",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/delegation.rs",
        PLUGIN_IMPORTER_ROWS_ROW_DATA_GUARD_NAME,
    ),
    (
        "row_children",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_children.rs",
        "assert_plugin_importer_row_data_children_are_current",
    ),
    (
        "row_data_status",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_data_status.rs",
        "assert_plugin_importer_row_data_owner_status_row_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/status_mirrors.rs",
        "assert_plugin_importer_row_data_status_mirrors_are_current",
    ),
];

#[test]
fn runtime_15_plugin_importer_rows_status_output_guard_is_folder_backed() {
    let route_source = read_runtime_src(PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_PATH);
    for (module_name, path, guard_name) in PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_CHILDREN {
        let module_mount = format!("#[path = \"plugin_importer_rows/{module_name}.rs\"]");
        assert_contains_all(
            "plugin-importer status-output guard mounts folder-backed child",
            &route_source,
            &[module_mount.as_str()],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
    }
    for forbidden in [
        [
            "tests/runtime_absorption/code_review_findings/",
            "plugin_importer_dx.rs",
        ]
        .concat(),
        ["let runtime_15", "_plan"].concat(),
        ["let status", "_map"].concat(),
        ["let date", "_map"].concat(),
    ] {
        assert!(
            !route_source.contains(&forbidden),
            "plugin_importer_rows.rs should route detailed checks to child files"
        );
    }
}

fn plugin_importer_status_output_guard_child_source_blob() -> String {
    PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}
