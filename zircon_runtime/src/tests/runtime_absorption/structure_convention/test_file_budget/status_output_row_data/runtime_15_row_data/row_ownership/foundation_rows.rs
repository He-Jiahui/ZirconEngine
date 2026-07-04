use super::*;

#[test]
fn runtime_15_row_data_foundation_rows_are_child_owned() {
    let runtime_15_foundation =
        read_runtime_src(RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "Runtime 15 foundation row-data child delegates foundation row literals",
        &runtime_15_foundation,
        &[
            "#[path = \"foundation/core_rows.rs\"]",
            "mod core_rows;",
            "#[path = \"foundation/typed_error_runtime_rows.rs\"]",
            "mod typed_error_runtime_rows;",
            "#[path = \"foundation/typed_error_plugin_rows.rs\"]",
            "mod typed_error_plugin_rows;",
            "#[path = \"foundation/typed_error_scene_asset_rows.rs\"]",
            "mod typed_error_scene_asset_rows;",
            "pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
