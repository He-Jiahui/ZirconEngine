use super::*;

const STATUS_ROW_DATA_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";
const STATUS_ROW_DATA_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";
const STATUS_MIRROR_DOC_PATHS: &[&str] = &[
    "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/zircon_runtime/structure/module-convention.md",
];

fn assert_status_docs_include(label: &str, expected: &[&str]) {
    for path in STATUS_MIRROR_DOC_PATHS {
        let source = read_repo(path);
        assert_contains_all(&format!("{path} mirrors {label}"), &source, expected);
    }
}

pub(super) fn assert_asset_tests_row_data_status_is_current() {
    let status_map = read_runtime_src(STATUS_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_ROW_DATA_DATE_MAP_PATH);

    assert_contains_all(
        "M3 status map records asset-tests row-data child split",
        &status_map,
        &[
            ASSET_TESTS_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            ASSET_TESTS_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records asset-tests row-data child split",
        &date_map,
        &[ASSET_TESTS_ROW_DATA_CHILD_SPLIT_STATUS_NAME, "2026-07-07"],
    );
    assert_status_docs_include(
        ASSET_TESTS_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
        &[
            ASSET_TESTS_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            ASSET_TESTS_ROW_DATA_CHILD_SPLIT_STATUS_ID,
            ASSET_TESTS_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
        ],
    );
}

pub(super) fn assert_asset_tests_guard_status_is_current() {
    let status_map = read_runtime_src(STATUS_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_ROW_DATA_DATE_MAP_PATH);
    let row_data_owner = read_runtime_src(ASSET_BUDGET_ASSET_TESTS_ROW_DATA_OWNER_PATH);

    assert_contains_all(
        "asset-tests row-data owner records guard folder-backed split",
        &row_data_owner,
        &[
            ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_NAME,
            ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_ID,
            ASSET_TESTS_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "M3 status map records asset-tests guard folder-backed split",
        &status_map,
        &[
            ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_NAME,
            ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records asset-tests guard folder-backed split",
        &date_map,
        &[ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_NAME, "2026-07-07"],
    );
    assert_status_docs_include(
        ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_NAME,
        &[
            ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_NAME,
            ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_ID,
            ASSET_TESTS_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
