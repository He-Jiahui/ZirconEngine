use super::*;

pub(super) const ASSET_BUDGET_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "root_inventory",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/root_inventory.rs",
        "runtime_15_asset_budget_row_data_root_inventory_is_child_owned",
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/export_chain.rs",
        "runtime_15_asset_budget_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/status_mirrors.rs",
        "runtime_15_asset_budget_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "asset_tests",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/asset_tests.rs",
        ASSET_TESTS_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "naming_graphics_misc",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/naming_graphics_misc.rs",
        NAMING_GRAPHICS_MISC_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_asset_budget_row_data/budgets.rs",
        "runtime_15_asset_budget_row_data_child_budgets_stay_focused",
    ),
];

pub(super) const NAMING_GRAPHICS_MISC_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_rows",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_GUARD_CHILD_ROWS_PATH,
        "naming-graphics-misc row-data route mounts child row groups",
    ),
    (
        "export_chain",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_GUARD_EXPORT_CHAIN_PATH,
        "asset-budget parent exports naming-graphics-misc children",
    ),
    (
        "folder_backed",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_GUARD_FOLDER_BACKED_PATH,
        "naming-graphics-misc guard route mounts folder-backed children",
    ),
    (
        "status_mirrors",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_GUARD_STATUS_MIRRORS_PATH,
        "NAMING_GRAPHICS_MISC_GUARD_FOLDER_BACKED_STATUS_NAME",
    ),
];

pub(super) const ASSET_TESTS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_rows",
        ASSET_BUDGET_ASSET_TESTS_GUARD_CHILD_ROWS_PATH,
        "asset-tests row-data route mounts child row groups",
    ),
    (
        "export_chain",
        ASSET_BUDGET_ASSET_TESTS_GUARD_EXPORT_CHAIN_PATH,
        "asset-budget parent exports asset-tests children",
    ),
    (
        "folder_backed",
        ASSET_BUDGET_ASSET_TESTS_GUARD_FOLDER_BACKED_PATH,
        "asset-tests guard route mounts folder-backed children",
    ),
    (
        "status_mirrors",
        ASSET_BUDGET_ASSET_TESTS_GUARD_STATUS_MIRRORS_PATH,
        "ASSET_TESTS_GUARD_FOLDER_BACKED_STATUS_NAME",
    ),
];
