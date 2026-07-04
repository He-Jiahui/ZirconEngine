use super::*;

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
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_GUARD_NAME,
    ),
];

pub(super) const PRIORITY_OWNER_BUDGETS: &[(&str, &str, usize)] = &[
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs.rs",
        PRIORITY_GUARD_PATH,
        90,
    ),
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/root_paths.rs",
        ROOT_PATHS_PATH,
        90,
    ),
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/root_statuses.rs",
        ROOT_STATUSES_PATH,
        80,
    ),
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/root_child_rows.rs",
        ROOT_CHILD_ROWS_PATH,
        120,
    ),
    (
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs/root_source_blobs.rs",
        ROOT_SOURCE_BLOBS_PATH,
        80,
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
