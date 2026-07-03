use super::super::*;

#[path = "plugin_importer_dx_child_owners/budgets.rs"]
mod budgets;
#[path = "plugin_importer_dx_child_owners/child_ownership.rs"]
mod child_ownership;
#[path = "plugin_importer_dx_child_owners/delegation.rs"]
mod delegation;
#[path = "plugin_importer_dx_child_owners/source_inventory.rs"]
mod source_inventory;
#[path = "plugin_importer_dx_child_owners/status_docs.rs"]
mod status_docs;
#[path = "plugin_importer_dx_child_owners/status_mirrors.rs"]
mod status_mirrors;
#[path = "plugin_importer_dx_child_owners/structure_assertions.rs"]
mod structure_assertions;

pub(super) const STRUCTURE_GUARD_PARENT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/delegation.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/child_ownership.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_mirrors.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/budgets.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory.rs";
pub(super) const PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/delegation.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/child_ownership.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/status_mirrors.rs";
pub(super) const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk.rs";
pub(super) const STRUCTURE_GUARD_ROWS: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/plugin_importer_dx.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const SLICE: &str =
    "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split";
pub(super) const STATUS: &str =
    "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const DATE: &str = "2026-06-30";
pub(super) const GUARD: &str =
    "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_is_child_owner";
pub(super) const FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 plugin-importer DX structure guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS: &str =
    "runtime_15_plugin_importer_dx_structure_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_DATE: &str = "2026-07-03";
pub(super) const FOLDER_BACKED_GUARD: &str =
    "runtime_15_plugin_importer_dx_structure_guard_is_folder_backed";
pub(super) const FOLDER_BACKED_STATUS_GUARD: &str =
    "runtime_15_plugin_importer_dx_structure_guard_folder_backed_status_is_current";
pub(super) const BUDGET_GUARD: &str =
    "runtime_15_plugin_importer_dx_structure_guard_budgets_are_focused";
pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD,
        FOLDER_BACKED_GUARD,
    ),
    (
        "child_ownership",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "source_inventory",
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD,
        "runtime_15_plugin_importer_dx_source_inventory_is_child_owner",
    ),
    (
        "structure_assertions",
        PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
        "pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed",
    ),
    (
        "status_docs",
        PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD,
        "runtime_15_plugin_importer_dx_status_docs_are_child_owner",
    ),
    (
        "status_mirrors",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    (
        "budgets",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD,
        BUDGET_GUARD,
    ),
];

pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed() {
    structure_assertions::assert_plugin_importer_dx_child_owners_are_folder_backed();
}

pub(super) fn assert_plugin_importer_dx_line_budgets() {
    source_inventory::assert_plugin_importer_dx_line_budgets();
}

pub(super) fn plugin_importer_dx_review_guard_count() -> usize {
    source_inventory::plugin_importer_dx_review_guard_count()
}

pub(super) fn assert_plugin_importer_dx_status_docs_are_synced() {
    status_docs::assert_plugin_importer_dx_status_docs_are_synced();
}

pub(super) fn folder_backed_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}
