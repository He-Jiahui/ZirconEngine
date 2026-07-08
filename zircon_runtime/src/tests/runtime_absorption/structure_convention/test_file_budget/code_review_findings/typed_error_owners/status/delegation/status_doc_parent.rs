use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_status_doc_parent_delegates_children() {
    let status_docs_parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD);

    assert_contains_all(
        "typed-error status-doc parent delegates focused guard children",
        &status_docs_parent,
        &[
            "#[path = \"status/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"status/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"status/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"status/paths.rs\"]",
            "mod paths;",
            "#[path = \"status/sources.rs\"]",
            "mod sources;",
            "#[path = \"status/status_maps.rs\"]",
            "mod status_maps;",
            "#[path = \"status/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) use child_sources::*;",
            "pub(super) use paths::*;",
            "pub(super) use sources::*;",
            "pub(super) fn assert_typed_error_status_docs_are_synced",
            "doc_mirrors::assert_typed_error_status_doc_mirrors_are_synced",
            "status_maps::assert_typed_error_status_maps_are_synced",
        ],
    );
}
