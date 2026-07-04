use super::super::*;

#[path = "structure_guard_children/budgets.rs"]
mod budgets;
#[path = "structure_guard_children/delegation.rs"]
mod delegation;
#[path = "structure_guard_children/folder_backed_summary.rs"]
mod folder_backed_summary;
#[path = "structure_guard_children/plugin_importer.rs"]
mod plugin_importer;
#[path = "structure_guard_children/review_guard_groups.rs"]
mod review_guard_groups;
#[path = "structure_guard_children/root_child_rows.rs"]
mod root_child_rows;
#[path = "structure_guard_children/root_inventory.rs"]
mod root_inventory;
#[path = "structure_guard_children/root_paths.rs"]
mod root_paths;
#[path = "structure_guard_children/root_sources.rs"]
mod root_sources;
#[path = "structure_guard_children/root_statuses.rs"]
mod root_statuses;
#[path = "structure_guard_children/status_docs.rs"]
mod status_docs;
#[path = "structure_guard_children/typed_error.rs"]
mod typed_error;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;
pub(super) use root_statuses::*;

pub(super) fn review_guard_status_rows_source() -> String {
    super::status_docs::review_guard_status_rows_source()
}

pub(super) fn assert_nested_structure_children_are_mounted() {
    folder_backed_summary::assert_folder_backed_summary_structure_children_are_mounted();
    typed_error::assert_typed_error_structure_children_are_mounted();
}
