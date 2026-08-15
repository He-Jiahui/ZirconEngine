use super::super::*;

#[path = "plugin_importer_dx_owners/budgets.rs"]
mod budgets;
#[path = "plugin_importer_dx_owners/child_ownership.rs"]
mod child_ownership;
#[path = "plugin_importer_dx_owners/delegation.rs"]
mod delegation;
#[path = "plugin_importer_dx_owners/root_child_rows.rs"]
mod root_child_rows;
#[path = "plugin_importer_dx_owners/root_inventory.rs"]
mod root_inventory;
#[path = "plugin_importer_dx_owners/root_paths.rs"]
mod root_paths;
#[path = "plugin_importer_dx_owners/root_sources.rs"]
mod root_sources;
#[path = "plugin_importer_dx_owners/source_inventory.rs"]
mod source_inventory;
#[path = "plugin_importer_dx_owners/structure_assertions.rs"]
mod structure_assertions;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

pub(super) const GUARD: &str =
    "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_is_child_owner";
pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed() {
    structure_assertions::assert_plugin_importer_dx_child_owners_are_folder_backed();
}

pub(super) fn assert_plugin_importer_dx_line_budgets() {
    source_inventory::assert_plugin_importer_dx_line_budgets();
}

pub(super) fn plugin_importer_dx_review_guard_count() -> usize {
    source_inventory::plugin_importer_dx_review_guard_count()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn plugin_importer_dx_source_inventory_child_source_blob(
) -> String {
    source_inventory::plugin_importer_dx_source_inventory_child_source_blob()
}
