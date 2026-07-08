use super::super::super::*;

#[path = "structure/child_ownership.rs"]
mod child_ownership;
#[path = "structure/d13_sdk.rs"]
mod d13_sdk;
#[path = "structure/delegation.rs"]
mod delegation;
#[path = "structure/review_mounts.rs"]
mod review_mounts;
#[path = "structure/status_mirrors.rs"]
mod status_mirrors;

pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure_assertions.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/delegation.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/child_ownership.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/status_mirrors.rs";
pub(super) const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk.rs";
pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/structure_assertions.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/plugin_importer_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/plugin_importer_maps.rs";

pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_SPLIT_ID: &str =
    "runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_static_passed_cargo_deferred";

pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTION_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD,
        "runtime_15_plugin_importer_dx_structure_assertions_are_child_owner",
    ),
    (
        "review_mounts",
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD,
        "pub(super) fn assert_plugin_importer_dx_review_mounts_are_folder_backed",
    ),
    (
        "child_ownership",
        PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD,
        "runtime_15_plugin_importer_dx_structure_assertions_children_are_child_owned",
    ),
    (
        "d13_sdk",
        PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD,
        "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner",
    ),
    (
        "status_mirrors",
        PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD,
        "runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_status_is_current",
    ),
];

pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed() {
    review_mounts::assert_plugin_importer_dx_review_mounts_are_folder_backed();
    d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed();
}

pub(super) fn plugin_importer_dx_structure_assertion_child_sources() -> Vec<(&'static str, String)>
{
    PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTION_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn plugin_importer_dx_structure_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in plugin_importer_dx_structure_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
