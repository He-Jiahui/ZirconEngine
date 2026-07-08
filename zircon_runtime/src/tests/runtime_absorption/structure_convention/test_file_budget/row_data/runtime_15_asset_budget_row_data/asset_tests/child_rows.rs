use super::*;

const ASSET_TESTS_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "project_rows",
        ASSET_BUDGET_ASSET_TESTS_PROJECT_ROWS_PATH,
        "Runtime 15 M3 asset project zmeta test folder split",
    ),
    (
        "asset_resource_rows",
        ASSET_BUDGET_ASSET_TESTS_ASSET_RESOURCE_ROWS_PATH,
        "Runtime 15 M3 asset artifact store test folder split",
    ),
    (
        "ui_pipeline_rows",
        ASSET_BUDGET_ASSET_TESTS_UI_PIPELINE_ROWS_PATH,
        "Runtime 15 M3 asset UI test folder split",
    ),
    (
        "row_data_owner",
        ASSET_BUDGET_ASSET_TESTS_ROW_DATA_OWNER_PATH,
        ASSET_TESTS_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
    ),
];

pub(super) fn assert_asset_tests_child_rows_are_route_owned() {
    let asset_tests_route = read_runtime_src(ASSET_BUDGET_ASSET_TESTS_PATH);

    assert_contains_all(
        "asset-tests row-data route mounts child row groups",
        &asset_tests_route,
        &[
            "asset_tests/project_rows.rs",
            "asset_tests/asset_resource_rows.rs",
            "asset_tests/ui_pipeline_rows.rs",
            "asset_tests/row_data_owner.rs",
            "project_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "asset_resource_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "ui_pipeline_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 asset project zmeta test folder split",
        "Runtime 15 M3 asset artifact store test folder split",
        "Runtime 15 M3 asset pipeline manager test folder split",
    ] {
        assert!(
            !asset_tests_route.contains(moved_row),
            "asset_tests.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in ASSET_TESTS_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "asset-tests child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            asset_tests_route.contains(&format!("mod {module_name};")),
            "asset_tests.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 130,
            "{path} should stay below its focused row-data budget"
        );
    }
}
