use super::*;

pub(super) const RUNTIME_ROW_DATA_CHILD_ROWS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "foundation_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_FOUNDATION_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/foundation_rows.rs",
        "Runtime 15 M3 foundation row-data guard child-owner split",
        "FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "lock_poison_scene_script_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_LOCK_POISON_SCENE_SCRIPT_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/lock_poison_scene_script_rows.rs",
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split",
        "LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_support_priority_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_GUARD_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/status_support_priority_rows.rs",
        "Runtime 15 M3 status-support row-data guard folder-backed split",
        "STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "asset_budget_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ASSET_BUDGET_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/asset_budget_rows.rs",
        "Runtime 15 M3 asset-budget row-data guard folder-backed split",
        "ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
];

pub(super) fn assert_runtime_row_data_parent_delegates_to_children() {
    let runtime_row_data = read_runtime_src(PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ROWS_PATH);
    for (module_name, read_path, _, representative_row, export_name) in RUNTIME_ROW_DATA_CHILD_ROWS
    {
        let path_attr = format!("#[path = \"runtime_row_data/{module_name}.rs\"]");
        let export_const = format!("pub(super) const {export_name}");
        let child = read_runtime_src(read_path);
        assert_contains_all(
            "production guard runtime row-data parent delegates to child",
            &runtime_row_data,
            &[path_attr.as_str(), export_const.as_str()],
        );
        assert!(
            !runtime_row_data.contains(representative_row),
            "runtime_row_data.rs should route {representative_row} instead of owning it"
        );
        assert_contains_all(
            read_path,
            &child,
            &[
                "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
                *representative_row,
            ],
        );
    }
}
