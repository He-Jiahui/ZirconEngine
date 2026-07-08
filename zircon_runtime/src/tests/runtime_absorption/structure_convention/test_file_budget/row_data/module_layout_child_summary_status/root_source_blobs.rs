use super::*;

pub(super) fn child_summary_status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    blob.push_str(&read_runtime_src(ROOT_STATUSES_PATH));
    blob.push('\n');
    for (_, path, _) in CHILD_SUMMARY_STATUS_DOC_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
