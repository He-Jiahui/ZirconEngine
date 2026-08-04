use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    let child = read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD);
    let child_inventory = format!(
        "{}\n{}",
        read_runtime_src(TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILD_ROWS_CHILD),
        read_runtime_src(TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_PATHS_CHILD)
    );
    let child_tree = moved_guard_absence_child_source_blob();

    assert_contains_all(
        "typed-error structure assertions delegates moved-guard absence checks",
        &parent,
        &[
            "#[path = \"structure/moved_guard_absence.rs\"]",
            "mod moved_guard_absence;",
            "moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned",
        ],
    );
    assert_contains_all(
        "typed-error moved-guard absence parent mounts focused children",
        &child,
        &[
            "mod child_ownership;",
            "mod child_ownership_status;",
            "mod preserved_guards;",
            "mod parent_backflow;",
            "mod path_anchors;",
            "mod budgets;",
            "mod status_mirrors;",
            "mod root_paths;",
            "mod root_statuses;",
            "mod root_child_rows;",
            "mod root_sources;",
            "mod root_inventory;",
            "pub(super) fn assert_typed_error_moved_guards_stay_child_owned",
            "pub(super) use root_sources::*;",
        ],
    );
    assert!(
        !child.contains("fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner"),
        "moved_guard_absence.rs should delegate the child-owner test body to {TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNERSHIP_CHILD}"
    );
    assert!(
        !parent.contains("let typed_error_children = super::source_inventory"),
        "structure_assertions.rs should delegate typed-error child-source aggregation to moved_guard_absence.rs"
    );
    assert!(
        !parent.contains("PARENT_BACKFLOW_GUARDS"),
        "structure_assertions.rs should not retain the moved-guard backflow list"
    );
    assert_contains_all(
        "typed-error moved-guard absence children own delegated checks",
        &child_tree,
        &[
            "runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
            "runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_children_line_budgets_are_current",
        ],
    );
    for (module_name, child_path, anchor) in TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILDREN {
        let path_attr = format!("#[path = \"moved_guard_absence/{module_name}.rs\"]");
        assert!(
            child.contains(&path_attr),
            "typed-error moved-guard absence parent should mount {module_name}"
        );
        assert!(
            child_inventory.contains(child_path),
            "typed-error moved-guard absence child inventory should list {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error moved-guard absence child {child_path} should own anchor {anchor}"
        );
    }

    assert_typed_error_moved_guards_stay_child_owned();
    path_anchors::assert_typed_error_child_path_anchors_are_current();
    budgets::assert_typed_error_moved_guard_absence_line_budgets();
}
