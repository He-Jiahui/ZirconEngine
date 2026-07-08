use super::super::super::*;

#[path = "status/child_sources.rs"]
mod child_sources;
#[path = "status/delegation.rs"]
mod delegation;
#[path = "status/doc_mirrors.rs"]
mod doc_mirrors;
#[path = "status/paths.rs"]
mod paths;
#[path = "status/source_helper_ownership.rs"]
mod source_helper_ownership;
#[path = "status/source_helper_status.rs"]
mod source_helper_status;
#[path = "status/sources.rs"]
mod sources;
#[path = "status/status_maps.rs"]
mod status_maps;
#[path = "status/status_mirrors.rs"]
mod status_mirrors;

pub(super) use child_sources::*;
pub(super) use paths::*;
pub(super) use sources::*;

pub(super) fn status_doc_rows_for_structure() -> String {
    sources::typed_error_status_row_source()
}

pub(super) fn status_doc_status_map_for_structure() -> String {
    sources::typed_error_status_map_source()
}

pub(super) fn status_doc_date_map_for_structure() -> String {
    sources::typed_error_date_map_source()
}

pub(super) fn assert_typed_error_status_docs_are_synced() {
    let sources = typed_error_status_doc_sources();

    doc_mirrors::assert_typed_error_status_doc_mirrors_are_synced(&sources);
    status_maps::assert_typed_error_status_maps_are_synced(&sources);
}
