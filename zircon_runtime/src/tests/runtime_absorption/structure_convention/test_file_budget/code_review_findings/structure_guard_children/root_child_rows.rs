use super::*;

pub(super) const STRUCTURE_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/delegation.rs",
        "runtime_15_code_review_findings_structure_guard_children_are_mounted",
    ),
    (
        "review_guard_groups",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/review_guard_groups.rs",
        "runtime_15_code_review_findings_structure_guard_review_groups_are_child_owned",
    ),
    (
        "plugin_importer",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs",
        "runtime_15_code_review_findings_structure_guard_plugin_importer_is_child_owned",
    ),
    (
        "status_docs",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/status_docs.rs",
        "runtime_15_code_review_findings_structure_guard_status_docs_are_child_owned",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets.rs",
        "runtime_15_code_review_findings_structure_guard_children_folder_backed_status_is_current",
    ),
];

pub(super) const STRUCTURE_GUARD_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        STRUCTURE_GUARD_ROOT_PATHS_CHILD,
        "STRUCTURE_GUARD_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        STRUCTURE_GUARD_ROOT_STATUSES_CHILD,
        STRUCTURE_GUARD_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        STRUCTURE_GUARD_ROOT_CHILD_ROWS_CHILD,
        "STRUCTURE_GUARD_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        STRUCTURE_GUARD_ROOT_SOURCES_CHILD,
        "structure_guard_status_row_source",
    ),
    (
        "root_inventory",
        STRUCTURE_GUARD_ROOT_INVENTORY_CHILD,
        STRUCTURE_GUARD_ROOT_INVENTORY_GUARD,
    ),
];
