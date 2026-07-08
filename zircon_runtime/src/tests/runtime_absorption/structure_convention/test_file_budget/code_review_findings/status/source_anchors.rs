use super::super::super::*;

#[path = "source_anchors/native_typed_error.rs"]
mod native_typed_error;
#[path = "source_anchors/review_sources.rs"]
mod review_sources;
#[path = "source_anchors/runtime_surface.rs"]
mod runtime_surface;
#[path = "source_anchors/status_mirrors.rs"]
mod status_mirrors;
#[path = "source_anchors/structure_owners.rs"]
mod structure_owners;

pub(super) const SOURCE_ANCHOR_OWNER_GUARD: &str =
    "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner";
pub(super) const SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_NAME: &str =
    super::STATUS_DOC_SOURCE_ANCHORS_FOLDER_BACKED_SLICE;
pub(super) const SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_ID: &str =
    super::STATUS_DOC_SOURCE_ANCHORS_FOLDER_BACKED_STATUS;
pub(super) const SOURCE_ANCHORS_REVIEW_SOURCES_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/source_anchors/review_sources.rs";
pub(super) const SOURCE_ANCHORS_NATIVE_TYPED_ERROR_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/source_anchors/native_typed_error.rs";
pub(super) const SOURCE_ANCHORS_RUNTIME_SURFACE_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/source_anchors/runtime_surface.rs";
pub(super) const SOURCE_ANCHORS_STRUCTURE_OWNERS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/source_anchors/structure_owners.rs";
pub(super) const SOURCE_ANCHORS_STATUS_MIRRORS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/source_anchors/status_mirrors.rs";

pub(super) const SOURCE_ANCHOR_CHILDREN: &[&str] = &[
    SOURCE_ANCHORS_REVIEW_SOURCES_OWNER,
    SOURCE_ANCHORS_NATIVE_TYPED_ERROR_OWNER,
    SOURCE_ANCHORS_RUNTIME_SURFACE_OWNER,
    SOURCE_ANCHORS_STRUCTURE_OWNERS_OWNER,
    SOURCE_ANCHORS_STATUS_MIRRORS_OWNER,
];

pub(super) fn assert_code_review_findings_status_doc_source_anchors<'a>(
    sources: impl IntoIterator<Item = (&'a str, &'a str)>,
    status_doc_child_anchors: &[&str],
) {
    for (label, source) in sources {
        review_sources::assert_review_source_anchors(label, source);
        native_typed_error::assert_native_typed_error_source_anchors(label, source);
        runtime_surface::assert_runtime_surface_source_anchors(label, source);
        structure_owners::assert_structure_owner_source_anchors(label, source);
        assert_contains_all(label, source, status_doc_child_anchors);
    }
}

pub(super) fn assert_status_doc_source_anchor_children_are_mounted() {
    let parent = read_runtime_src(super::STATUS_DOC_SOURCE_ANCHORS_OWNER);
    assert_contains_all(
        "status-doc source anchors parent mounts folder-backed children",
        &parent,
        &[
            "mod review_sources;",
            "mod native_typed_error;",
            "mod runtime_surface;",
            "mod structure_owners;",
            "mod status_mirrors;",
            SOURCE_ANCHORS_REVIEW_SOURCES_OWNER,
            SOURCE_ANCHORS_NATIVE_TYPED_ERROR_OWNER,
            SOURCE_ANCHORS_RUNTIME_SURFACE_OWNER,
            SOURCE_ANCHORS_STRUCTURE_OWNERS_OWNER,
            SOURCE_ANCHORS_STATUS_MIRRORS_OWNER,
        ],
    );
    for moved_anchor in [
        concat!(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/",
            "asset_loaders/animation_binary.rs"
        ),
        concat!(
            "review_f5_native_live_host_registration_replay_",
            "uses_typed_error"
        ),
        concat!(
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/",
            "d13_importer_sdk/runtime_manifests.rs"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "source_anchors.rs should delegate source anchor `{moved_anchor}` to child files"
        );
    }
}

pub(super) fn status_doc_source_anchor_child_sources() -> Vec<(&'static str, String)> {
    SOURCE_ANCHOR_CHILDREN
        .iter()
        .map(|path| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn status_doc_source_anchor_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in status_doc_source_anchor_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
