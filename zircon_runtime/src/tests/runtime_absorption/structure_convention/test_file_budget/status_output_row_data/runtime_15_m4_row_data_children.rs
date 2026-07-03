use super::*;

#[path = "runtime_15_m4_row_data_children/budgets.rs"]
mod budgets;
#[path = "runtime_15_m4_row_data_children/delegation.rs"]
mod delegation;
#[path = "runtime_15_m4_row_data_children/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_m4_row_data_children/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const M4_ROW_DATA_CHILDREN_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M4_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs";
pub(super) const RUNTIME_15_M4_CORE_RHI_DYNAMIC_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/core_rhi_dynamic.rs";
pub(super) const RUNTIME_15_M4_ASSET_SCENE_RENDER_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/asset_scene_render.rs";
pub(super) const RUNTIME_15_M4_UI_TEXT_TEMPLATE_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_text_template.rs";
pub(super) const RUNTIME_15_M4_UI_SURFACE_PLUGIN_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_surface_plugin.rs";
pub(super) const RUNTIME_15_M4_ROW_DATA_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/row_data_owner.rs";
pub(super) const RUNTIME_15_M4_RENDER_SHADER_SYNC_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/render_shader_sync.rs";
pub(super) const M4_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs";
pub(super) const M4_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs";

pub(super) const ROW_DATA_OWNER_STATUS_NAME: &str = "Runtime 15 M4 row-data owner child split";
pub(super) const ROW_DATA_OWNER_STATUS_ID: &str =
    "runtime_15_m4_row_data_owner_child_split_static_passed_cargo_deferred";
pub(super) const ROW_DATA_OWNER_GUARD_NAME: &str = "runtime_15_m4_row_data_owner_is_child_backed";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 M4 row-data children guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_m4_row_data_children_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_m4_row_data_children_guard_is_folder_backed";

pub(super) const M4_ROW_DATA_CHILDREN_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/row_ownership.rs",
        ROW_DATA_OWNER_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/status_mirrors.rs",
        "runtime_15_m4_row_data_children_guard_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/budgets.rs",
        "runtime_15_m4_row_data_children_guard_children_stay_focused",
    ),
];

pub(super) const M4_ROW_DATA_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
        RUNTIME_15_M4_EXPECTED_STATUS_ROW_DATA_PATH,
        120,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/core_rhi_dynamic.rs",
        RUNTIME_15_M4_CORE_RHI_DYNAMIC_ROW_DATA_PATH,
        160,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/asset_scene_render.rs",
        RUNTIME_15_M4_ASSET_SCENE_RENDER_ROW_DATA_PATH,
        180,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_text_template.rs",
        RUNTIME_15_M4_UI_TEXT_TEMPLATE_ROW_DATA_PATH,
        180,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_surface_plugin.rs",
        RUNTIME_15_M4_UI_SURFACE_PLUGIN_ROW_DATA_PATH,
        160,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/row_data_owner.rs",
        RUNTIME_15_M4_ROW_DATA_OWNER_PATH,
        100,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/render_shader_sync.rs",
        RUNTIME_15_M4_RENDER_SHADER_SYNC_ROW_DATA_PATH,
        120,
    ),
];

pub(super) fn m4_row_data_children_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in M4_ROW_DATA_CHILDREN_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
