use super::*;

#[test]
fn runtime_15_priority_plan_docs_row_data_guard_children_stay_focused() {
    for (label, path, budget) in PRIORITY_OWNER_BUDGETS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= *budget,
            "{label} should stay below its priority-plan-doc owner budget of {budget} lines; got {line_count}"
        );
    }

    for (_, child_path, _) in PRIORITY_ROW_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 120,
            "{child_path} should stay below the priority-plan-doc row-data child budget; got {line_count}"
        );
    }
    for (_, child_path, _) in PRIORITY_OWNER_GUARD_ROW_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 120,
            "{child_path} should stay below the priority-plan-doc owner-guard row-data child budget; got {line_count}"
        );
    }
    for (_, child_path, _) in PRIORITY_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 160,
            "{child_path} should stay focused after priority-plan-doc guard folder-backed split; got {line_count}"
        );
    }
}
