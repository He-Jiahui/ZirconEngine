use super::*;

pub(super) const EVIDENCE_ANCHORS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors/delegation.rs",
        FOLDER_BACKED_GUARD_NAME,
    ),
    (
        "variable_evidence",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors/variable_evidence.rs",
        VARIABLE_EVIDENCE_GUARD_NAME,
    ),
    (
        "root_inventory",
        ROOT_INVENTORY_GUARD_PATH,
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors/status_mirrors.rs",
        "runtime_15_status_output_evidence_anchors_folder_backed_status_mirrors_are_current",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors/budgets.rs",
        "runtime_15_status_output_evidence_anchors_guard_children_stay_focused",
    ),
];
