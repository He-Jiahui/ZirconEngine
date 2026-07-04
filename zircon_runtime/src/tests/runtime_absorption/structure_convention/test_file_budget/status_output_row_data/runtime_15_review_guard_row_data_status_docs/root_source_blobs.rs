use super::*;

pub(super) fn status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_DOC_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
