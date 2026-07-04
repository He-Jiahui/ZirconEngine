use super::super::super::*;

#[path = "status_docs/child_sources.rs"]
mod child_sources;
#[path = "status_docs/delegation.rs"]
mod delegation;
#[path = "status_docs/doc_mirrors.rs"]
mod doc_mirrors;
#[path = "status_docs/paths.rs"]
mod paths;
#[path = "status_docs/source_helper_ownership.rs"]
mod source_helper_ownership;
#[path = "status_docs/source_helper_status.rs"]
mod source_helper_status;
#[path = "status_docs/sources.rs"]
mod sources;
#[path = "status_docs/status_maps.rs"]
mod status_maps;
#[path = "status_docs/status_mirrors.rs"]
mod status_mirrors;

pub(super) use child_sources::*;
pub(super) use paths::*;
pub(super) use sources::*;

pub(super) fn assert_typed_error_status_docs_are_synced() {
    let sources = typed_error_status_doc_sources();

    doc_mirrors::assert_typed_error_status_doc_mirrors_are_synced(&sources);
    status_maps::assert_typed_error_status_maps_are_synced(&sources);
}
