use super::*;

#[test]
fn runtime_15_foundation_row_data_exports_are_child_owned() {
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_foundation =
        read_runtime_src(RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "Runtime 15 root delegates foundation rows",
        &runtime_15,
        &[
            "#[path = \"runtime_15/foundation.rs\"]",
            "mod foundation;",
            "pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "foundation::FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES",
            "foundation::FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES",
            "foundation::FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
            "foundation::FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "top-level status row data aggregation keeps every Runtime 15 foundation topic group",
        &parent,
        &[
            "runtime_15::RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for foundation_row in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 F5 UI input surrounding-text error source",
    ] {
        assert!(
            !runtime_15.contains(foundation_row),
            "expected_status_row_data/runtime_15.rs should delegate foundation row literal {foundation_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 foundation child mounts topic row owners",
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
    for moved_row in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 F5 UI input surrounding-text error source",
        "Runtime 15 F5 native plugin descriptor ABI typed errors",
        "Runtime 15 F5 asset meta typed errors",
    ] {
        assert!(
            !runtime_15_foundation.contains(moved_row),
            "foundation.rs should delegate row literal {moved_row} to a topic child owner"
        );
    }
}
