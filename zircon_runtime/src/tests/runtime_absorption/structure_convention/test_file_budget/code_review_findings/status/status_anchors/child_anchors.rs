use super::super::super::super::*;
use super::*;

#[path = "child_anchors/aggregation.rs"]
mod aggregation;
#[path = "child_anchors/boundary_samples.rs"]
mod boundary_samples;
#[path = "child_anchors/folder_backed_summary.rs"]
mod folder_backed_summary;
#[path = "child_anchors/inventory.rs"]
mod inventory;
#[path = "child_anchors/source_blob.rs"]
mod source_blob;
#[path = "child_anchors/split_layout.rs"]
mod split_layout;
#[path = "child_anchors/status_docs.rs"]
mod status_docs;
#[path = "child_anchors/structure_guards.rs"]
mod structure_guards;

pub(super) use aggregation::status_doc_child_anchors;
pub(super) use inventory::{
    STATUS_DOC_CHILD_ANCHOR_CHILDREN, STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_DATE,
    STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_GUARD, STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_ID,
    STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_NAME,
    STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_DATE,
    STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_GUARD,
    STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_ID,
    STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_NAME,
};
pub(super) use source_blob::status_doc_child_anchor_child_source_blob;

pub(super) fn status_doc_review_guard_status_rows_source() -> String {
    super::super::review_guard_status_rows_source()
}

pub(super) fn status_doc_review_guard_status_map_source() -> String {
    super::super::review_guard_status_map_source()
}

pub(super) fn status_doc_review_guard_date_map_source() -> String {
    super::super::review_guard_date_map_source()
}
