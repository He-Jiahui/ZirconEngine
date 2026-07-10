use super::*;

#[test]
fn runtime_15_review_guard_typed_error_row_data_status_mirrors_are_current() {
    let status_rows = review_guard_status_support_review_rows_source_blob();
    let status_map = read_runtime_src_route_tree(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH);
    let date_map = read_runtime_src_route_tree(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH);
    let docs = [
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
        read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        read_repo("docs/plans/engine-code-structure-convention.md"),
        read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        read_repo("docs/zircon_runtime/structure/module-convention.md"),
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
    ]
    .join("\n");

    let status_anchors = [
        TYPED_ERROR_ROW_DATA_STATUS_NAME,
        TYPED_ERROR_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/native_plugin_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/runtime_surface_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/asset_shader_rows.rs",
        TYPED_ERROR_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard support rows record typed-error row-data split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "review status/date maps record typed-error row-data split",
        &format!("{status_map}\n{date_map}"),
        &[
            TYPED_ERROR_ROW_DATA_STATUS_NAME,
            TYPED_ERROR_ROW_DATA_STATUS_ID,
            "2026-07-04",
        ],
    );
    assert_contains_all(
        "typed-error row-data split is mirrored in docs and session state",
        &docs,
        &status_anchors,
    );
}
