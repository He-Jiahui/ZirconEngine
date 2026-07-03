use super::*;

#[test]
fn runtime_15_scene_script_row_data_child_budgets_stay_focused() {
    for (label, path, budget) in SCENE_SCRIPT_ROW_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its child-owner budget of {budget} lines; got {line_count}"
        );
    }

    for (_, child_path, _) in SCENE_SCRIPT_ROW_DATA_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{child_path} should stay focused after scene-script row-data guard folder-backed split; got {line_count} lines"
        );
    }
}
