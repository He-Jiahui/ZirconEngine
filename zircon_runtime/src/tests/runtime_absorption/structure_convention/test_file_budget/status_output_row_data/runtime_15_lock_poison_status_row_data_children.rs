use super::*;

#[path = "runtime_15_lock_poison_status_row_data_children/budgets.rs"]
mod budgets;
#[path = "runtime_15_lock_poison_status_row_data_children/delegation.rs"]
mod delegation;
#[path = "runtime_15_lock_poison_status_row_data_children/export_chain.rs"]
mod export_chain;
#[path = "runtime_15_lock_poison_status_row_data_children/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_lock_poison_status_row_data_children/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const LOCK_POISON_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs";
pub(super) const LOCK_POISON_STATUS_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/status_rows.rs";
pub(super) const LOCK_POISON_STATUS_POLICY_GUARDS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/policy_guards.rs";
pub(super) const LOCK_POISON_STATUS_CORE_RUNTIME_RECOVERY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/core_runtime_recovery.rs";
pub(super) const LOCK_POISON_STATUS_RUNTIME_SERVICES_RECOVERY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/runtime_services_recovery.rs";
pub(super) const LOCK_POISON_STATUS_RESOURCE_RENDER_INPUT_RECOVERY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/resource_render_input_recovery.rs";
pub(super) const LOCK_POISON_STATUS_SCRIPT_VM_RECOVERY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/script_vm_recovery.rs";
pub(super) const LOCK_POISON_STATUS_ROW_DATA_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/row_data_owner.rs";
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
    "Runtime 15 M3 lock-poison status row-data owner child split";
pub(super) const CHILD_OWNER_STATUS_ID: &str =
    "runtime_15_lock_poison_status_row_data_owner_child_split_static_passed_cargo_deferred";
pub(super) const CHILD_OWNER_GUARD_NAME: &str =
    "runtime_15_lock_poison_status_row_data_owner_is_child_backed";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 lock-poison status row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_lock_poison_status_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_lock_poison_status_row_data_children_guard_is_folder_backed";

pub(super) const LOCK_POISON_STATUS_ROW_DATA_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_lock_poison_status_row_data_children/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_lock_poison_status_row_data_children/row_ownership.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_lock_poison_status_row_data_children/export_chain.rs",
        "runtime_15_lock_poison_status_row_data_export_chain_is_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_lock_poison_status_row_data_children/status_mirrors.rs",
        "runtime_15_lock_poison_status_row_data_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_lock_poison_status_row_data_children/budgets.rs",
        "runtime_15_lock_poison_status_row_data_child_budgets_stay_focused",
    ),
];

pub(super) const LOCK_POISON_ROW_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
        LOCK_POISON_STATUS_ROWS_PATH,
        140,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/status_rows.rs",
        LOCK_POISON_STATUS_STATUS_ROWS_PATH,
        90,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/policy_guards.rs",
        LOCK_POISON_STATUS_POLICY_GUARDS_PATH,
        90,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/core_runtime_recovery.rs",
        LOCK_POISON_STATUS_CORE_RUNTIME_RECOVERY_PATH,
        110,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/runtime_services_recovery.rs",
        LOCK_POISON_STATUS_RUNTIME_SERVICES_RECOVERY_PATH,
        100,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/resource_render_input_recovery.rs",
        LOCK_POISON_STATUS_RESOURCE_RENDER_INPUT_RECOVERY_PATH,
        110,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/script_vm_recovery.rs",
        LOCK_POISON_STATUS_SCRIPT_VM_RECOVERY_PATH,
        80,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/row_data_owner.rs",
        LOCK_POISON_STATUS_ROW_DATA_OWNER_PATH,
        90,
    ),
];

pub(super) fn lock_poison_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in LOCK_POISON_STATUS_ROW_DATA_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
