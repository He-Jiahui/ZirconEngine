use super::super::super::super::super::super::super::*;
use super::super::super::child_inventory::TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN;

pub(in super::super::super) fn typed_error_status_doc_paths_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super::super) fn typed_error_status_doc_paths_child_source_blob() -> String {
    let mut blob = String::new();
    for (path, source) in typed_error_status_doc_paths_child_sources() {
        blob.push_str(path);
        blob.push('\n');
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
