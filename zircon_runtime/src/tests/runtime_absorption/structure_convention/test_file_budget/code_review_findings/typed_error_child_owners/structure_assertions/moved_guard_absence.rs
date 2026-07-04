use super::super::super::super::*;

#[path = "moved_guard_absence/budgets.rs"]
mod budgets;
#[path = "moved_guard_absence/parent_backflow.rs"]
mod parent_backflow;
#[path = "moved_guard_absence/path_anchors.rs"]
mod path_anchors;
#[path = "moved_guard_absence/preserved_guards.rs"]
mod preserved_guards;
#[path = "moved_guard_absence/root_child_rows.rs"]
mod root_child_rows;
#[path = "moved_guard_absence/root_inventory.rs"]
mod root_inventory;
#[path = "moved_guard_absence/root_paths.rs"]
mod root_paths;
#[path = "moved_guard_absence/root_sources.rs"]
mod root_sources;
#[path = "moved_guard_absence/root_statuses.rs"]
mod root_statuses;
#[path = "moved_guard_absence/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;
pub(super) use root_statuses::*;

pub(super) fn assert_typed_error_moved_guards_stay_child_owned() {
    preserved_guards::assert_typed_error_preserved_review_guards_are_current();
    parent_backflow::assert_typed_error_parent_backflow_guards_are_absent();
}

#[test]
fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    let child = read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD);
    let child_inventory = read_runtime_src(TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILD_ROWS_CHILD);
    let child_tree = moved_guard_absence_child_source_blob();

    assert_contains_all(
        "typed-error structure assertions delegates moved-guard absence checks",
        &parent,
        &[
            "#[path = \"structure_assertions/moved_guard_absence.rs\"]",
            "mod moved_guard_absence;",
            "moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned",
        ],
    );
    assert_contains_all(
        "typed-error moved-guard absence parent mounts focused children",
        &child,
        &[
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
            "moved_guard_absence_child_source_blob",
        ],
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
            "runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_children_line_budgets_are_current",
            "runtime_15_typed_error_moved_guard_absence_guard_folder_backed_status_is_current",
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
