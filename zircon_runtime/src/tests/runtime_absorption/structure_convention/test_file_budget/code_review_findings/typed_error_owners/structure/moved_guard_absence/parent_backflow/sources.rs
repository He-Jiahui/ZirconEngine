use super::super::super::super::super::super::*;
use super::*;

pub(super) fn typed_error_parent_backflow_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_parent_backflow_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in typed_error_parent_backflow_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn typed_error_parent_backflow_parent_sources(
) -> Vec<(&'static str, &'static str, String)> {
    TYPED_ERROR_PARENT_PATHS
        .iter()
        .map(|(label, path)| (*label, *path, read_runtime_src(path)))
        .collect()
}
