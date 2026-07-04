use super::super::*;

#[path = "plugin_importer_dx_child_owners/budgets.rs"]
mod budgets;
#[path = "plugin_importer_dx_child_owners/child_ownership.rs"]
mod child_ownership;
#[path = "plugin_importer_dx_child_owners/delegation.rs"]
mod delegation;
#[path = "plugin_importer_dx_child_owners/root_child_rows.rs"]
mod root_child_rows;
#[path = "plugin_importer_dx_child_owners/root_inventory.rs"]
mod root_inventory;
#[path = "plugin_importer_dx_child_owners/root_paths.rs"]
mod root_paths;
#[path = "plugin_importer_dx_child_owners/root_sources.rs"]
mod root_sources;
#[path = "plugin_importer_dx_child_owners/root_statuses.rs"]
mod root_statuses;
#[path = "plugin_importer_dx_child_owners/source_inventory.rs"]
mod source_inventory;
#[path = "plugin_importer_dx_child_owners/status_docs.rs"]
mod status_docs;
#[path = "plugin_importer_dx_child_owners/status_mirrors.rs"]
mod status_mirrors;
#[path = "plugin_importer_dx_child_owners/structure_assertions.rs"]
mod structure_assertions;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;
pub(super) use root_statuses::*;

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
