use super::*;

#[test]
fn runtime_15_foundation_row_data_child_budgets_stay_focused() {
    for path in [
        RUNTIME_15_ROW_DATA_GUARD_PATH,
        FOUNDATION_ROW_DATA_GUARD_PATH,
        TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH,
        RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH,
        RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH,
        FOUNDATION_CORE_ROWS_PATH,
        FOUNDATION_TYPED_ERROR_RUNTIME_ROWS_PATH,
        FOUNDATION_TYPED_ERROR_PLUGIN_ROWS_PATH,
        FOUNDATION_TYPED_ERROR_SCENE_ASSET_ROWS_PATH,
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (_, child_path, _) in FOUNDATION_ROW_DATA_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 220,
            "{child_path} should stay focused after Runtime 15 foundation row-data folder-backed split; got {line_count} lines"
        );
    }
}
