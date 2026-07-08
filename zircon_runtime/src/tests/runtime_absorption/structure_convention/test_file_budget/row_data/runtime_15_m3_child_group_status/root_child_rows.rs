use super::*;

pub(super) const STATUS_DOCS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/budgets.rs",
        "runtime_15_m3_child_group_status_docs_child_budgets_stay_focused",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "source_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/source_ownership.rs",
        "runtime_15_m3_child_group_status_doc_sources_are_child_owned",
    ),
    (
        "status_maps",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/status_maps.rs",
        "runtime_15_m3_child_group_status_doc_maps_are_current",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status/status_mirrors.rs",
        HISTORICAL_GUARD_NAME,
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
];
