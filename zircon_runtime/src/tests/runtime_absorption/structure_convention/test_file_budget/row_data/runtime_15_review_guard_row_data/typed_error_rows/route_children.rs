use super::*;

#[test]
fn runtime_15_review_guard_typed_error_rows_are_child_owned() {
    let typed_error_rows = read_runtime_src(TYPED_ERROR_ROWS_PATH);

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
}
