use super::*;

pub(super) const FOUNDATION_GUARDS_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "root_inventory",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/root_inventory.rs",
        "runtime_15_foundation_guards_row_data_root_inventory_is_child_owned",
    ),
    (
        "runtime_structure_tests",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/runtime_structure_tests.rs",
        RUNTIME_STRUCTURE_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/export_chain.rs",
        "runtime_15_foundation_guards_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/status_mirrors.rs",
        "runtime_15_foundation_guards_row_data_guard_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/budgets.rs",
        "runtime_15_foundation_guards_row_data_child_budgets_stay_focused",
    ),
];

pub(super) const RUNTIME_STRUCTURE_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_rows",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_GUARD_CHILD_ROWS_PATH,
        "runtime-structure row-data route mounts child row groups",
    ),
    (
        "export_chain",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_GUARD_EXPORT_CHAIN_PATH,
        "foundation-guards parent exports runtime-structure children",
    ),
    (
        "folder_backed",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_GUARD_FOLDER_BACKED_PATH,
        "runtime-structure guard route mounts folder-backed children",
    ),
    (
        "status_mirrors",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_GUARD_STATUS_MIRRORS_PATH,
        "RUNTIME_STRUCTURE_GUARD_FOLDER_BACKED_STATUS_NAME",
    ),
];
