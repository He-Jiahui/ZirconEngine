use super::*;

pub(in super::super) fn status_doc_child_anchor_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path) in STATUS_DOC_CHILD_ANCHOR_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
