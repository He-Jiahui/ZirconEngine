use super::super::super::super::*;
use super::*;

#[path = "child_anchors/folder_backed_summary.rs"]
mod folder_backed_summary;
#[path = "child_anchors/status_docs.rs"]
mod status_docs;
#[path = "child_anchors/structure_guards.rs"]
mod structure_guards;

pub(super) const STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings status-doc child-anchor list child split";
pub(super) const STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_ID: &str =
    "runtime_15_code_review_findings_status_docs_child_anchor_list_child_split_static_passed_cargo_deferred";
pub(super) const STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_DATE: &str = "2026-07-05";
pub(super) const STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_GUARD: &str =
    "runtime_15_code_review_findings_status_docs_child_anchor_list_is_child_owned";

pub(super) const STATUS_DOC_CHILD_ANCHOR_CHILDREN: &[(&str, &str)] = &[
    (
        "folder_backed_summary",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors/child_anchors/folder_backed_summary.rs",
    ),
    (
        "structure_guards",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors/child_anchors/structure_guards.rs",
    ),
    (
        "status_docs",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors/child_anchors/status_docs.rs",
    ),
];

pub(super) fn status_doc_child_anchors() -> Vec<&'static str> {
    let mut anchors = Vec::new();
    anchors.extend_from_slice(folder_backed_summary::STATUS_DOC_FOLDER_BACKED_SUMMARY_ANCHORS);
    anchors.extend_from_slice(structure_guards::STATUS_DOC_STRUCTURE_GUARD_ANCHORS);
    anchors.extend_from_slice(status_docs::STATUS_DOC_SELF_ANCHORS);
    anchors
}

pub(super) fn status_doc_child_anchor_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path) in STATUS_DOC_CHILD_ANCHOR_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

fn status_doc_child_anchor_boundary_samples() -> Vec<&'static str> {
    let mut anchors = Vec::new();
    anchors.extend_from_slice(
        folder_backed_summary::STATUS_DOC_FOLDER_BACKED_SUMMARY_BOUNDARY_ANCHORS,
    );
    anchors.extend_from_slice(structure_guards::STATUS_DOC_STRUCTURE_GUARD_BOUNDARY_ANCHORS);
    anchors.extend_from_slice(status_docs::STATUS_DOC_SELF_BOUNDARY_ANCHORS);
    anchors
}

#[test]
fn runtime_15_code_review_findings_status_docs_child_anchor_list_is_child_owned() {
    let parent = read_runtime_src(STATUS_DOC_STATUS_CHILD_ANCHORS_OWNER);
    let child_blob = status_doc_child_anchor_child_source_blob();
    let anchors = status_doc_child_anchors();

    for (_, child_path) in STATUS_DOC_CHILD_ANCHOR_CHILDREN {
        assert!(
            parent.contains(child_path),
            "status-doc child-anchor route should inventory child path {child_path}"
        );
    }
    for moved_anchor in status_doc_child_anchor_boundary_samples() {
        assert!(
            !parent.contains(moved_anchor),
            "child_anchors.rs should delegate concrete child anchor `{moved_anchor}` to child files"
        );
        assert!(
            child_blob.contains(moved_anchor),
            "child-anchor child files should own concrete child anchor `{moved_anchor}`"
        );
        assert!(
            anchors.contains(&moved_anchor),
            "status_doc_child_anchors() should aggregate concrete child anchor `{moved_anchor}`"
        );
    }
    assert!(anchors
        .contains(&"runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner"));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_NAME));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_ID));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_GUARD));
}
