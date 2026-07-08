use super::super::super::super::super::super::super::*;
use super::super::super::child_inventory::TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_CHILDREN;

pub(in super::super::super) fn typed_error_status_mirror_status_current_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super::super) fn typed_error_status_mirror_status_current_child_source_blob() -> String
{
    let mut blob = String::new();
    for (_, source) in typed_error_status_mirror_status_current_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
