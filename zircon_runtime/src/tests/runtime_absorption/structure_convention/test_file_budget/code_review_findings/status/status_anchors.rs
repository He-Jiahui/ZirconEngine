use super::super::super::*;

#[path = "status_anchors/child_anchors.rs"]
mod child_anchors;
#[path = "status_anchors/map_anchors.rs"]
mod map_anchors;

pub(super) const STATUS_DOC_STATUS_CHILD_ANCHORS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchors/child_anchors.rs";
pub(super) const STATUS_DOC_STATUS_MAP_ANCHORS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchors/map_anchors.rs";
pub(super) const STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings status-doc status anchors folder-backed split";
pub(super) const STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_ID: &str =
    "runtime_15_code_review_findings_status_docs_status_anchors_folder_backed_static_passed_cargo_deferred";

pub(super) const STATUS_DOC_MAP_ANCHORS: &[&str] = map_anchors::STATUS_DOC_MAP_ANCHORS;

pub(super) const STATUS_DOC_STATUS_ANCHOR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_anchors",
        STATUS_DOC_STATUS_CHILD_ANCHORS_OWNER,
        "runtime_15_code_review_findings_status_docs_status_child_anchors_are_child_owned",
    ),
    (
        "map_anchors",
        STATUS_DOC_STATUS_MAP_ANCHORS_OWNER,
        "runtime_15_code_review_findings_status_docs_status_map_anchors_are_child_owned",
    ),
];

pub(super) fn status_doc_child_anchors() -> Vec<&'static str> {
    child_anchors::status_doc_child_anchors()
}

pub(super) fn status_doc_session_anchors() -> Vec<&'static str> {
    status_doc_child_anchors()
}

pub(super) fn status_doc_status_anchor_child_sources() -> Vec<(&'static str, String)> {
    STATUS_DOC_STATUS_ANCHOR_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn status_doc_status_anchor_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in status_doc_status_anchor_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob.push_str(&child_anchors::status_doc_child_anchor_child_source_blob());
    blob
}
