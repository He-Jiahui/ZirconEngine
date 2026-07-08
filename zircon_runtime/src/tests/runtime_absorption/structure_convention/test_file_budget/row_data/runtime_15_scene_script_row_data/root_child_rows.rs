use super::*;

pub(super) const SCENE_SCRIPT_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "root_inventory",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/root_inventory.rs",
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/export_chain.rs",
        "runtime_15_scene_script_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/status_mirrors.rs",
        "runtime_15_scene_script_row_data_status_mirror_children_are_child_owned",
    ),
    (
        "runtime_07_performance",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/runtime_07_performance.rs",
        RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "plugin_extension_tests",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/plugin_extension_tests.rs",
        PLUGIN_EXTENSION_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/budgets.rs",
        "runtime_15_scene_script_row_data_child_budgets_stay_focused",
    ),
];

pub(super) const RUNTIME_07_PERFORMANCE_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_rows",
        SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_GUARD_CHILD_ROWS_PATH,
        "Runtime 07 performance row-data route mounts child row groups",
    ),
    (
        "export_chain",
        SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_GUARD_EXPORT_CHAIN_PATH,
        "scene-script parent exports Runtime 07 performance children",
    ),
    (
        "folder_backed",
        SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_PATH,
        "Runtime 07 performance guard route mounts folder-backed children",
    ),
    (
        "status_mirrors",
        SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_GUARD_STATUS_MIRRORS_PATH,
        "RUNTIME_07_PERFORMANCE_GUARD_FOLDER_BACKED_STATUS_NAME",
    ),
];

pub(super) const PLUGIN_EXTENSION_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_rows",
        SCENE_SCRIPT_PLUGIN_EXTENSION_GUARD_CHILD_ROWS_PATH,
        "plugin-extension row-data route mounts child row groups",
    ),
    (
        "export_chain",
        SCENE_SCRIPT_PLUGIN_EXTENSION_GUARD_EXPORT_CHAIN_PATH,
        "scene-script parent exports plugin-extension children",
    ),
    (
        "folder_backed",
        SCENE_SCRIPT_PLUGIN_EXTENSION_GUARD_FOLDER_BACKED_PATH,
        "plugin-extension guard route mounts folder-backed children",
    ),
    (
        "status_mirrors",
        SCENE_SCRIPT_PLUGIN_EXTENSION_GUARD_STATUS_MIRRORS_PATH,
        "PLUGIN_EXTENSION_GUARD_FOLDER_BACKED_STATUS_NAME",
    ),
];
