use super::super::super::super::*;

#[path = "native_plugin_loader/budgets.rs"]
mod budgets;
#[path = "native_plugin_loader/child_inventory.rs"]
mod child_inventory;
#[path = "native_plugin_loader/delegation.rs"]
mod delegation;
#[path = "native_plugin_loader/metadata.rs"]
mod metadata;
#[path = "native_plugin_loader/routes.rs"]
mod routes;
#[path = "native_plugin_loader/source_helper_ownership.rs"]
mod source_helper_ownership;
#[path = "native_plugin_loader/source_helper_status.rs"]
mod source_helper_status;
#[path = "native_plugin_loader/sources.rs"]
mod sources;
#[path = "native_plugin_loader/status_mirrors.rs"]
mod status_mirrors;

pub(super) use child_inventory::*;
pub(super) use metadata::*;
pub(super) use sources::*;

pub(super) fn assert_typed_error_native_plugin_loader_children_are_folder_backed() {
    let sources = typed_error_native_plugin_loader_sources();
    routes::assert_typed_error_native_plugin_loader_routes_are_folder_backed(&sources);
}

#[test]
fn runtime_15_typed_error_native_plugin_loader_structure_is_child_owner() {
    let sources = typed_error_native_plugin_loader_sources();

    delegation::assert_typed_error_native_plugin_loader_structure_is_child_owner(&sources);
    routes::assert_typed_error_native_plugin_loader_routes_are_folder_backed(&sources);
    budgets::assert_typed_error_native_plugin_loader_structure_budgets_are_focused(&sources);
}
