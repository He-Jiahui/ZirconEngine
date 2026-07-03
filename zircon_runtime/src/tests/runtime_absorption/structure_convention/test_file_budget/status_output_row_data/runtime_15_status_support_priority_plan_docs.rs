use super::*;

#[path = "runtime_15_status_support_priority_plan_docs/budgets.rs"]
mod budgets;
#[path = "runtime_15_status_support_priority_plan_docs/delegation.rs"]
mod delegation;
#[path = "runtime_15_status_support_priority_plan_docs/export_chain.rs"]
mod export_chain;
#[path = "runtime_15_status_support_priority_plan_docs/row_sources.rs"]
mod row_sources;
#[path = "runtime_15_status_support_priority_plan_docs/status_mirrors.rs"]
mod status_mirrors;
pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const PRIORITY_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs.rs";
pub(super) const PRIORITY_ROW_PARENT: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs";
pub(super) const RUNTIME_15_M3_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const RUNTIME_15_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const TOP_LEVEL_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const PRIORITY_ROW_DATA_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const HISTORICAL_STATUS_NAME: &str =
    "Runtime 15 M3 priority plan docs row-data owner child split";
pub(super) const HISTORICAL_STATUS_ID: &str =
    "runtime_15_priority_plan_docs_row_data_owner_child_split_static_passed_cargo_deferred";
pub(super) const HISTORICAL_GUARD_NAME: &str =
    "runtime_15_priority_plan_docs_row_data_owner_is_child_backed";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 priority plan docs row-data guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_priority_plan_docs_row_data_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_priority_plan_docs_row_data_guard_is_folder_backed";
pub(super) const OWNER_GUARD_CHILD_STATUS_NAME: &str =
    "Runtime 15 M3 priority plan docs owner-guard row-data child split";
pub(super) const OWNER_GUARD_CHILD_STATUS_ID: &str =
    "runtime_15_priority_plan_docs_owner_guard_row_data_child_split_static_passed_cargo_deferred";
pub(super) const OWNER_GUARD_CHILD_GUARD_NAME: &str =
    "runtime_15_priority_plan_docs_owner_guard_rows_are_child_owned";
pub(super) const PRIORITY_ROW_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "integrity_guards",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs",
        "Runtime 15 M3 priority plan docs code-path integrity guard",
    ),
    (
        "owner_guards",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs",
        "inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_followups",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs",
        "Runtime 15 M3 status output owner stale-path follow-up",
    ),
    (
        "row_data_owner",
        PRIORITY_ROW_DATA_OWNER_PATH,
        HISTORICAL_STATUS_NAME,
    ),
];
pub(super) const PRIORITY_OWNER_GUARD_ROW_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "layout_rows",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/layout_rows.rs",
        "Runtime 15 M3 priority plan docs guard child-owner split",
    ),
    (
        "inventory_rows",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/inventory_rows.rs",
        "Runtime 15 M3 priority plan docs moved guard path mirror",
    ),
];
pub(super) const PRIORITY_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/budgets.rs",
        "runtime_15_priority_plan_docs_row_data_guard_children_stay_focused",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "export_chain",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/export_chain.rs",
        "runtime_15_priority_plan_docs_row_data_export_chain_is_current",
    ),
    (
        "row_sources",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/row_sources.rs",
        HISTORICAL_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/status_mirrors.rs",
        "runtime_15_priority_plan_docs_row_data_guard_folder_backed_status_mirrors_are_current",
    ),
];
pub(super) const PRIORITY_OWNER_BUDGETS: &[(&str, &str, usize)] = &[
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs.rs",
        PRIORITY_GUARD_PATH,
        140,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs",
        PRIORITY_ROW_PARENT,
        90,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs",
        PRIORITY_ROW_DATA_OWNER_PATH,
        120,
    ),
];
pub(super) fn priority_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in PRIORITY_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
