use super::*;

#[test]
fn runtime_15_row_data_owner_budgets_are_child_owned() {
    for (label, path, budget) in RUNTIME_15_ROW_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below the Runtime 15 row-data budget; got {line_count} lines"
        );
    }
}
