use super::*;

pub(super) const STATUS_ROW_DOCS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "row_sources",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/row_sources.rs",
        CHILD_OWNER_GUARD_NAME,
    ),
    (
        "status_maps",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_maps.rs",
        "runtime_15_m3_child_group_status_row_doc_maps_are_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors.rs",
        "runtime_15_m3_child_group_status_row_doc_folder_backed_status_mirrors_are_current",
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_row_docs/budgets.rs",
        "runtime_15_m3_child_group_status_row_doc_child_budgets_stay_focused",
    ),
];
