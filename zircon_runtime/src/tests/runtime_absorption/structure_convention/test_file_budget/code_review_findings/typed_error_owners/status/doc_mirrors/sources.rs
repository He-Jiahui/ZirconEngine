use super::super::super::super::super::*;
use super::*;

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
        .chain(TYPED_ERROR_STATUS_DOC_MIRROR_SOURCE_HELPER_CHILDREN.iter())
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_status_doc_mirror_child_source_blob() -> String {
    source_blob_from(typed_error_status_doc_mirror_child_sources())
}

pub(super) fn typed_error_status_doc_mirror_source_helper_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOC_MIRROR_SOURCE_HELPER_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_status_doc_mirror_source_helper_child_source_blob() -> String {
    source_blob_from(typed_error_status_doc_mirror_source_helper_child_sources())
}

fn source_blob_from(sources: Vec<(&'static str, String)>) -> String {
    let mut blob = String::new();
    for (path, source) in sources {
        blob.push_str(path);
        blob.push('\n');
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
