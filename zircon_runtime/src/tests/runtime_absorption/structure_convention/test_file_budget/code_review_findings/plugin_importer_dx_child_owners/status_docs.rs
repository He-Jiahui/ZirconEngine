use super::super::super::*;

#[path = "status_docs/delegation.rs"]
mod delegation;
#[path = "status_docs/doc_mirrors.rs"]
mod doc_mirrors;
#[path = "status_docs/root_child_rows.rs"]
mod root_child_rows;
#[path = "status_docs/root_inventory.rs"]
mod root_inventory;
#[path = "status_docs/root_paths.rs"]
mod root_paths;
#[path = "status_docs/root_sources.rs"]
mod root_sources;
#[path = "status_docs/root_statuses.rs"]
mod root_statuses;
#[path = "status_docs/status_maps.rs"]
mod status_maps;
#[path = "status_docs/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;
pub(super) use root_statuses::*;

#[test]
fn runtime_15_plugin_importer_dx_status_docs_are_child_owner() {
    assert_plugin_importer_dx_status_docs_are_synced();
}

pub(super) fn assert_plugin_importer_dx_status_docs_are_synced() {
    let sources = plugin_importer_dx_status_doc_sources();

    doc_mirrors::assert_plugin_importer_dx_status_doc_mirrors_are_synced(&sources);
    status_maps::assert_plugin_importer_dx_status_maps_are_synced(&sources);
}
