use super::super::super::super::*;

#[path = "moved_guard_absence/budgets.rs"]
mod budgets;
#[path = "moved_guard_absence/parent_backflow.rs"]
mod parent_backflow;
#[path = "moved_guard_absence/path_anchors.rs"]
mod path_anchors;
#[path = "moved_guard_absence/preserved_guards.rs"]
mod preserved_guards;
#[path = "moved_guard_absence/status_mirrors.rs"]
mod status_mirrors;

const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs";
const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence.rs";
const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PRESERVED_GUARDS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/preserved_guards.rs";
const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/parent_backflow.rs";
const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PATH_ANCHORS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/path_anchors.rs";
const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/budgets.rs";
const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_STATUS_MIRRORS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/status_mirrors.rs";
const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs";
const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

const TYPED_ERROR_MOVED_GUARD_ABSENCE_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split";
const TYPED_ERROR_MOVED_GUARD_ABSENCE_FOLDER_BACKED_STATUS: &str =
    "runtime_15_typed_error_moved_guard_absence_guard_folder_backed_static_passed_cargo_deferred";

const TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILDREN: &[(&str, &str, &str)] = &[
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
    (
        "status_mirrors",
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_STATUS_MIRRORS_CHILD,
        "runtime_15_typed_error_moved_guard_absence_guard_folder_backed_status_is_current",
    ),
];

pub(super) fn assert_typed_error_moved_guards_stay_child_owned() {
    preserved_guards::assert_typed_error_preserved_review_guards_are_current();
    parent_backflow::assert_typed_error_parent_backflow_guards_are_absent();
}

fn typed_error_children_source() -> String {
    super::super::source_inventory::typed_error_children_source()
}

fn moved_guard_absence_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

fn moved_guard_absence_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in moved_guard_absence_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

fn review_guard_status_rows_source() -> String {
    read_runtime_src(REVIEW_GUARD_STATUS_ROWS_PATH)
}

#[test]
fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    let child = read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD);
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
            "#[path = \"moved_guard_absence/preserved_guards.rs\"]",
            "mod preserved_guards;",
            "#[path = \"moved_guard_absence/parent_backflow.rs\"]",
            "mod parent_backflow;",
            "#[path = \"moved_guard_absence/path_anchors.rs\"]",
            "mod path_anchors;",
            "#[path = \"moved_guard_absence/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"moved_guard_absence/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) fn assert_typed_error_moved_guards_stay_child_owned",
            "moved_guard_absence_child_sources",
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
    for (_, child_path, anchor) in TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILDREN {
        assert!(
            child.contains(child_path),
            "typed-error moved-guard absence parent should inventory child path {child_path}"
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
