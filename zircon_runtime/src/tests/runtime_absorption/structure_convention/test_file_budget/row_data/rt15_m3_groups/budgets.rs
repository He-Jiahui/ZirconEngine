use super::*;

#[test]
fn runtime_15_m3_child_groups_row_data_guard_children_stay_focused() {
    for group in M3_CHILD_GROUP_OWNER_PATH_GROUPS {
        for (label, path, budget) in *group {
            let source = read_runtime_src(path);
            let line_count = source.lines().count();
            assert!(
                line_count < *budget,
                "{label} should stay below its owner budget of {budget} lines; got {line_count}"
            );
        }
    }
    for (label, path, budget) in ROOT_PATHS_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its root path owner budget of {budget} lines; got {line_count}"
        );
    }
    for (label, path, budget) in PRODUCTION_GUARD_CORE_AND_EVIDENCE_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its core-and-evidence row-data owner budget of {budget} lines; got {line_count}"
        );
    }
    for (label, path, budget) in PRODUCTION_GUARD_RUNTIME_ROW_DATA_GUARD_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its production guard runtime row-data guard owner budget of {budget} lines; got {line_count}"
        );
    }
    for (label, path, budget) in PRODUCTION_GUARD_REVIEW_GUARD_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its review-guard row-data owner budget of {budget} lines; got {line_count}"
        );
    }
    for (label, path, budget) in PRODUCTION_GUARD_MODULE_LAYOUT_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its module-layout row-data owner budget of {budget} lines; got {line_count}"
        );
    }
    for (label, path, budget) in PRODUCTION_GUARD_STATUS_DOCS_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its status-doc row-data owner budget of {budget} lines; got {line_count}"
        );
    }
    for (label, path, budget) in MODULE_CONVENTION_STATUS_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its module-convention status owner budget of {budget} lines; got {line_count}"
        );
    }
    for (label, path, budget) in RUNTIME_15_EXPORT_OWNER_PATHS {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{label} should stay below its Runtime 15 export owner budget of {budget} lines; got {line_count}"
        );
    }

    for (_, child_path, _) in M3_CHILD_GROUP_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{child_path} should stay focused after M3 child-groups guard folder-backed split; got {line_count} lines"
        );
    }
}
