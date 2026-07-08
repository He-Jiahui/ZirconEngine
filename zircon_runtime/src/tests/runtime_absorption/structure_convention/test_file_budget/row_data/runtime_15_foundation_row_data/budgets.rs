use super::*;

#[test]
fn runtime_15_foundation_row_data_child_budgets_stay_focused() {
    for (label, path, max_lines) in FOUNDATION_ROW_DATA_GUARD_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *max_lines,
            "{label} at {path} should stay below {max_lines} lines; got {line_count}"
        );
    }
}
