use super::*;

pub(super) const TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_ownership",
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNERSHIP_CHILD,
        "runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
    ),
    (
        "preserved_guards",
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PRESERVED_GUARDS_CHILD,
        "runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned",
    ),
    (
        "parent_backflow",
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILD,
        "runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned",
    ),
    (
        "path_anchors",
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PATH_ANCHORS_CHILD,
        "runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned",
    ),
    (
        "budgets",
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_BUDGETS_CHILD,
        "runtime_15_typed_error_moved_guard_absence_children_line_budgets_are_current",
    ),
];

pub(super) const TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_PATHS_CHILD,
        "TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_PATHS_CHILD",
    ),
    (
        "root_child_rows",
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILD_ROWS_CHILD,
        "TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_SOURCES_CHILD,
        "moved_guard_absence_child_sources",
    ),
];
