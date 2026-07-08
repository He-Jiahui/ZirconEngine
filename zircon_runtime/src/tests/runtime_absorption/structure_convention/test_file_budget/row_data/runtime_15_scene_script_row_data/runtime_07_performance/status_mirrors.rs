use super::*;

const OTHER_ROW_DATA_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/other_row_data_maps.rs";
const OTHER_ROW_DATA_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/other_row_data_maps.rs";
const STATUS_MIRROR_DOC_PATHS: &[&str] = &[
    "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/zircon_runtime/structure/module-convention.md",
    ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
];

fn assert_status_docs_include(label: &str, expected: &[&str]) {
    for path in STATUS_MIRROR_DOC_PATHS {
        let source = read_repo(path);
        assert_contains_all(&format!("{path} mirrors {label}"), &source, expected);
    }
}

pub(super) fn assert_runtime_07_performance_row_data_status_is_current() {
    let status_map = read_runtime_src(OTHER_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(OTHER_ROW_DATA_DATE_MAP_PATH);

    assert_contains_all(
        "M3 status map records Runtime 07 performance row-data child split",
        &status_map,
        &[
            RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records Runtime 07 performance row-data child split",
        &date_map,
        &[
            RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            "2026-07-07",
        ],
    );
    assert_status_docs_include(
        RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
        &[
            RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_STATUS_ID,
            RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
        ],
    );
}

pub(super) fn assert_runtime_07_performance_guard_status_is_current() {
    let status_map = read_runtime_src(OTHER_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(OTHER_ROW_DATA_DATE_MAP_PATH);
    let row_data_owner = read_runtime_src(SCENE_SCRIPT_RUNTIME_07_ROW_DATA_OWNER_PATH);

    assert_contains_all(
        "Runtime 07 performance row-data owner records guard folder-backed split",
        &row_data_owner,
        &[
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_NAME,
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_ID,
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "M3 status map records Runtime 07 performance guard folder-backed split",
        &status_map,
        &[
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_NAME,
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records Runtime 07 performance guard folder-backed split",
        &date_map,
        &[
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
    assert_status_docs_include(
        RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_NAME,
        &[
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_NAME,
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_ID,
            RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
