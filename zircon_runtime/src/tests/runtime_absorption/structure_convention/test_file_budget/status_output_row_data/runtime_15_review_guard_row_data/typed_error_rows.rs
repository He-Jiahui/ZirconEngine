use super::*;

#[test]
fn runtime_15_review_guard_typed_error_rows_are_child_owned() {
    let typed_error_rows = read_runtime_src(TYPED_ERROR_ROWS_PATH);
    let native_plugin_rows = read_runtime_src(TYPED_ERROR_NATIVE_PLUGIN_ROWS_PATH);
    let runtime_surface_rows = read_runtime_src(TYPED_ERROR_RUNTIME_SURFACE_ROWS_PATH);
    let asset_shader_rows = read_runtime_src(TYPED_ERROR_ASSET_SHADER_ROWS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let status_rows = read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH);
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let docs = [
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
        read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        read_repo("docs/plans/engine-code-structure-convention.md"),
        read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        read_repo("docs/zircon_runtime/structure/module-convention.md"),
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
    ]
    .join("\n");

    assert_contains_all(
        "typed-error row-data parent routes to child owners",
        &typed_error_rows,
        &[
            "#[path = \"typed_error_rows/native_plugin_rows.rs\"]",
            "#[path = \"typed_error_rows/runtime_surface_rows.rs\"]",
            "#[path = \"typed_error_rows/asset_shader_rows.rs\"]",
            "native_plugin_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_surface_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "asset_shader_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "TYPED_ERROR_ROW_ANCHOR_MIRROR",
        ],
    );
    assert!(
        !typed_error_rows.contains("pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &["),
        "typed_error_rows.rs should route child row-data owners instead of owning row tuples directly"
    );

    let child_rows = [
        native_plugin_rows.as_str(),
        runtime_surface_rows.as_str(),
        asset_shader_rows.as_str(),
    ]
    .join("\n");
    assert_contains_all(
        "typed-error row-data children own representative rows",
        &child_rows,
        &[
            "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
            "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split",
            "Runtime 15 M3 scene world typed-error review guard child-owner split",
            "Runtime 15 M3 UI input typed-error review guard child-owner split",
            "Runtime 15 M3 asset records typed-error review guard child-owner split",
            "Runtime 15 M3 shader prewarm CLI typed-error review guard child-owner split",
        ],
    );

    assert_contains_all(
        "typed-error row groups are exported through the status-output chain",
        &(review_guard_splits + runtime_15_m3.as_str() + runtime_15.as_str() + top_level.as_str()),
        &[
            "TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
            "REVIEW_GUARD_TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "REVIEW_GUARD_TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );

    let status_anchors = [
        TYPED_ERROR_ROW_DATA_STATUS_NAME,
        TYPED_ERROR_ROW_DATA_STATUS_ID,
        TYPED_ERROR_ROWS_PATH,
        TYPED_ERROR_NATIVE_PLUGIN_ROWS_PATH,
        TYPED_ERROR_RUNTIME_SURFACE_ROWS_PATH,
        TYPED_ERROR_ASSET_SHADER_ROWS_PATH,
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
        &(status_map + date_map.as_str()),
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
