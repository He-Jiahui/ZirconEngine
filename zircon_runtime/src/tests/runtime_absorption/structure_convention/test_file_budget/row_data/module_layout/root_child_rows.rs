use super::*;

pub(super) const MODULE_LAYOUT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout/delegation.rs",
        HISTORICAL_GUARD_NAME,
    ),
    (
        "child_summaries",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout/child_summaries.rs",
        "runtime_15_status_output_row_data_module_layout_child_summaries_stay_delegated",
    ),
    (
        "root_inventory",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout/root_inventory.rs",
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout/status_mirrors.rs",
        "runtime_15_status_output_row_data_module_layout_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout/budgets.rs",
        "runtime_15_status_output_row_data_module_layout_children_stay_focused",
    ),
];
