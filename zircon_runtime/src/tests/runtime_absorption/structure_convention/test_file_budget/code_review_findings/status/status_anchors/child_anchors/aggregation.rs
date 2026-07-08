use super::{folder_backed_summary, status_docs, structure_guards};

pub(in super::super) fn status_doc_child_anchors() -> Vec<&'static str> {
    let mut anchors = Vec::new();
    anchors.extend_from_slice(folder_backed_summary::STATUS_DOC_FOLDER_BACKED_SUMMARY_ANCHORS);
    anchors.extend_from_slice(structure_guards::STATUS_DOC_STRUCTURE_GUARD_ANCHORS);
    anchors.extend_from_slice(status_docs::STATUS_DOC_SELF_ANCHORS);
    anchors
}
