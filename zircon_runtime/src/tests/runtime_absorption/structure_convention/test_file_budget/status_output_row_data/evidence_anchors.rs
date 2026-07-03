use super::*;

#[path = "evidence_anchors/budgets.rs"]
mod budgets;
#[path = "evidence_anchors/delegation.rs"]
mod delegation;
#[path = "evidence_anchors/status_mirrors.rs"]
mod status_mirrors;
#[path = "evidence_anchors/variable_evidence.rs"]
mod variable_evidence;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const EVIDENCE_ANCHORS_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs";
pub(super) const EXPECTED_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_rows.rs";
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs";
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs";
pub(super) const RUNTIME_15_M4_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs";
pub(super) const RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
pub(super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

pub(super) const VARIABLE_EVIDENCE_STATUS_NAME: &str =
    "Runtime 15 M3 status output variable evidence anchors";
pub(super) const VARIABLE_EVIDENCE_STATUS_ID: &str =
    "runtime_15_status_output_variable_evidence_anchors_static_passed_cargo_deferred";
pub(super) const VARIABLE_EVIDENCE_GUARD_NAME: &str =
    "runtime_15_expected_status_output_rows_accept_variable_evidence_anchors";
pub(super) const FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 status output evidence anchors guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS_ID: &str =
    "runtime_15_status_output_evidence_anchors_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_GUARD_NAME: &str =
    "runtime_15_status_output_evidence_anchors_guard_is_folder_backed";

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

pub(super) const EVIDENCE_ANCHOR_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs",
        EVIDENCE_ANCHORS_GUARD_PATH,
        100,
    ),
    (
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
        RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_ROW_DATA_PATH,
        700,
    ),
];

pub(super) fn evidence_anchors_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in EVIDENCE_ANCHORS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
