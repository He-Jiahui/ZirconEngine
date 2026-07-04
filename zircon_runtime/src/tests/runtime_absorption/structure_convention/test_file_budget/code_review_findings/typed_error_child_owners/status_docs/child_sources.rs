use super::super::super::super::*;
use super::*;

pub(super) fn typed_error_status_docs_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN
        .iter()
        .chain(TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN.iter())
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_status_docs_child_source_blob() -> String {
    source_blob_from(typed_error_status_docs_child_sources())
}

pub(super) fn typed_error_status_doc_source_helper_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_status_doc_source_helper_child_source_blob() -> String {
    source_blob_from(typed_error_status_doc_source_helper_child_sources())
}

fn source_blob_from(sources: Vec<(&'static str, String)>) -> String {
    let mut blob = String::new();
    for (_, source) in sources {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
