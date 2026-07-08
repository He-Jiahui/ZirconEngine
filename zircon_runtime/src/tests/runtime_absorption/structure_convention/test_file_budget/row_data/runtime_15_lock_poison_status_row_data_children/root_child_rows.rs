use super::*;

pub(super) const LOCK_POISON_STATUS_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "root_inventory",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/root_inventory.rs",
        "runtime_15_lock_poison_status_row_data_root_inventory_is_child_owned",
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/export_chain.rs",
        "runtime_15_lock_poison_status_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/status_mirrors.rs",
        "runtime_15_lock_poison_status_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_lock_poison_status_row_data_children/budgets.rs",
        "runtime_15_lock_poison_status_row_data_child_budgets_stay_focused",
    ),
];
