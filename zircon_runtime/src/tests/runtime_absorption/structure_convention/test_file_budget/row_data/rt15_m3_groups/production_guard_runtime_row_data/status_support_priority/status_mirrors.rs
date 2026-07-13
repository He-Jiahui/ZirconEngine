use super::*;

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

pub(super) fn assert_status_support_priority_row_data_status_is_current() {
    let status_map =
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH);

    assert_contains_all(
        "M3 status map records status-support priority child split",
        &status_map,
        &[
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_STATUS_NAME,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records status-support priority child split",
        &date_map,
        &[
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_STATUS_NAME,
            "2026-07-07",
        ],
    );
    assert_status_docs_include(
        PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_STATUS_NAME,
        &[
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_STATUS_NAME,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_STATUS_ID,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_GUARD_NAME,
        ],
    );
}

pub(super) fn assert_status_support_priority_guard_status_is_current() {
    let status_map =
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH);
    let row_data_owner = read_runtime_src(
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_OWNER_ROWS_PATH,
    );

    assert_contains_all(
        "status-support priority row-data owner records guard folder-backed split",
        &row_data_owner,
        &[
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_NAME,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_ID,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "M3 status map records status-support priority guard folder-backed split",
        &status_map,
        &[
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_NAME,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records status-support priority guard folder-backed split",
        &date_map,
        &[
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
    assert_status_docs_include(
        PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_NAME,
        &[
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_NAME,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_ID,
            PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
