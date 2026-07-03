use super::*;

#[path = "runtime_15_scene_script_row_data/budgets.rs"]
mod budgets;
#[path = "runtime_15_scene_script_row_data/delegation.rs"]
mod delegation;
#[path = "runtime_15_scene_script_row_data/export_chain.rs"]
mod export_chain;
#[path = "runtime_15_scene_script_row_data/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_scene_script_row_data/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const SCENE_SCRIPT_TESTS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs";
pub(super) const SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs";
pub(super) const SCENE_SCRIPT_SCRIPT_VM_RUNTIME_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_runtime.rs";
pub(super) const SCENE_SCRIPT_PLUGIN_EXTENSION_TESTS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs";
pub(super) const SCENE_SCRIPT_GAMEPLAY_SHADER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_gameplay_shader.rs";
pub(super) const SCENE_SCRIPT_SCENE_ECS_TESTS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_ecs_tests.rs";
pub(super) const SCENE_SCRIPT_SCENE_ASSET_WORLD_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_asset_world.rs";
pub(super) const SCENE_SCRIPT_ROW_DATA_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/row_data_owner.rs";
pub(super) const M3_STRUCTURE_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs";
pub(super) const M3_STRUCTURE_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";

pub(super) const CHILD_OWNER_STATUS_NAME: &str =
    "Runtime 15 M3 scene-script row-data owner child split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_scene_script_row_data_owner_child_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_scene_script_row_data_owner_is_child_backed";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 scene-script row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_scene_script_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_scene_script_row_data_guard_is_folder_backed";

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
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/export_chain.rs",
        "runtime_15_scene_script_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/status_mirrors.rs",
        "runtime_15_scene_script_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/budgets.rs",
        "runtime_15_scene_script_row_data_child_budgets_stay_focused",
    ),
];

pub(super) const SCENE_SCRIPT_ROW_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
        SCENE_SCRIPT_TESTS_ROWS_PATH,
        140,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs",
        SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_PATH,
        90,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_runtime.rs",
        SCENE_SCRIPT_SCRIPT_VM_RUNTIME_PATH,
        90,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs",
        SCENE_SCRIPT_PLUGIN_EXTENSION_TESTS_PATH,
        140,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_gameplay_shader.rs",
        SCENE_SCRIPT_GAMEPLAY_SHADER_PATH,
        90,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_ecs_tests.rs",
        SCENE_SCRIPT_SCENE_ECS_TESTS_PATH,
        110,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_asset_world.rs",
        SCENE_SCRIPT_SCENE_ASSET_WORLD_PATH,
        90,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/row_data_owner.rs",
        SCENE_SCRIPT_ROW_DATA_OWNER_PATH,
        90,
    ),
];

pub(super) fn scene_script_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in SCENE_SCRIPT_ROW_DATA_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
