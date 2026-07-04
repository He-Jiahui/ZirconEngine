use super::*;

pub(super) const SCENE_SCRIPT_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "root_inventory",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/root_inventory.rs",
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/export_chain.rs",
        "runtime_15_scene_script_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/status_mirrors.rs",
        "runtime_15_scene_script_row_data_status_mirror_children_are_child_owned",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/budgets.rs",
        "runtime_15_scene_script_row_data_child_budgets_stay_focused",
    ),
];
