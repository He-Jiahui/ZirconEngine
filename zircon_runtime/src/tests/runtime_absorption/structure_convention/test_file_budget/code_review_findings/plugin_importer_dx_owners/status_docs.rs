use super::super::super::*;

#[path = "status/delegation.rs"]
mod delegation;
#[path = "status/doc_mirrors.rs"]
mod doc_mirrors;
#[path = "status/root_child_rows.rs"]
mod root_child_rows;
#[path = "status/root_inventory.rs"]
mod root_inventory;
#[path = "status/root_paths.rs"]
mod root_paths;
#[path = "status/root_sources.rs"]
mod root_sources;
#[path = "status/root_statuses.rs"]
mod root_statuses;
#[path = "status/status_maps.rs"]
mod status_maps;
#[path = "status/status_mirrors.rs"]
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

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn plugin_importer_dx_status_docs_structure_source_blob(
) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER,
        read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER),
        plugin_importer_dx_status_docs_child_source_blob(),
        read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_STATUSES_OWNER),
    )
}
