use super::super::super::super::*;
use super::*;

#[path = "doc_mirrors/guard_anchors.rs"]
mod guard_anchors;
#[path = "doc_mirrors/source_paths.rs"]
mod source_paths;
#[path = "doc_mirrors/status_current.rs"]
mod status_current;
#[path = "doc_mirrors/status_slices.rs"]
mod status_slices;

pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors.rs";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_SLICES_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/status_slices.rs";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_PATHS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/source_paths.rs";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_GUARD_ANCHORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/guard_anchors.rs";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_CURRENT_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/status_current.rs";

pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 typed-error status-doc doc mirrors folder-backed split";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_STATUS: &str =
    "runtime_15_typed_error_status_doc_mirrors_folder_backed_static_passed_cargo_deferred";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_typed_error_status_doc_mirrors_are_folder_backed";
pub(super) const TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_GUARD: &str =
    "runtime_15_typed_error_status_doc_mirrors_folder_backed_status_is_current";

pub(super) const TYPED_ERROR_STATUS_DOC_MIRROR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "status_slices",
        TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_SLICES_CHILD,
        "assert_typed_error_status_doc_slice_anchors_are_synced",
    ),
    (
        "source_paths",
        TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_PATHS_CHILD,
        "assert_typed_error_status_doc_source_paths_are_synced",
    ),
    (
        "guard_anchors",
        TYPED_ERROR_STATUS_DOC_MIRRORS_GUARD_ANCHORS_CHILD,
        "assert_typed_error_status_doc_guard_anchors_are_synced",
    ),
    (
        "status_current",
        TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_CURRENT_CHILD,
        TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_GUARD,
    ),
];

pub(super) fn assert_typed_error_status_doc_mirrors_are_synced(
    sources: &TypedErrorStatusDocSources,
) {
    status_slices::assert_typed_error_status_doc_slice_anchors_are_synced(sources);
    source_paths::assert_typed_error_status_doc_source_paths_are_synced(sources);
    guard_anchors::assert_typed_error_status_doc_guard_anchors_are_synced(sources);
}

#[test]
fn runtime_15_typed_error_status_doc_mirrors_are_folder_backed() {
    let sources = typed_error_status_doc_sources();
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOC_MIRRORS_CHILD);
    let child_tree = typed_error_status_doc_mirror_child_source_blob();

    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOC_MIRROR_CHILDREN {
        assert!(
            parent.contains(child_path),
            "typed-error status-doc mirror parent should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc mirror child {child_path} should own anchor {anchor}"
        );
    }
    assert_typed_error_status_doc_mirrors_are_synced(&sources);
}

pub(super) fn typed_error_status_doc_mirror_sources<'a>(
    sources: &'a TypedErrorStatusDocSources,
) -> [(&'static str, &'a str); 6] {
    [
        ("Runtime 15 plan", sources.runtime_15_plan.as_str()),
        ("Runtime index", sources.runtime_index.as_str()),
        ("review findings", sources.review_findings.as_str()),
        (
            "structure convention",
            sources.structure_convention.as_str(),
        ),
        ("module convention doc", sources.module_doc.as_str()),
        ("status-output row data", sources.status_rows.as_str()),
    ]
}

pub(super) fn typed_error_status_doc_mirror_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOC_MIRROR_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_status_doc_mirror_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in typed_error_status_doc_mirror_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
