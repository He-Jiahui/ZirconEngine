use super::*;

#[test]
fn runtime_15_m4_row_data_children_guard_children_stay_focused() {
    for (label, path, budget) in M4_ROW_DATA_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its child-owner budget of {budget} lines; got {line_count}"
        );
    }

    for (_, child_path, _) in M4_ROW_DATA_CHILDREN_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{child_path} should stay focused after M4 row-data children guard folder-backed split; got {line_count} lines"
        );
    }
}
