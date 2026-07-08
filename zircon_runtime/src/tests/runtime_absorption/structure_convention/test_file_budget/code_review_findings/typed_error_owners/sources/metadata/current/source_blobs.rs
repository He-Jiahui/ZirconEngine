use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

pub(in super::super) fn metadata_child_source_blob() -> String {
    source_blob_from(metadata_child_sources())
}

pub(in super::super) fn metadata_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn metadata_status_current_child_source_blob() -> String {
    source_blob_from(metadata_status_current_child_sources())
}

pub(in super::super) fn metadata_status_current_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
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
