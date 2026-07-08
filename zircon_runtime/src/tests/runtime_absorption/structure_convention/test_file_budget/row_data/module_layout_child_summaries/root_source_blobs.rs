use super::*;

pub(super) fn child_summary_child_sources() -> Vec<(&'static str, String)> {
    CHILD_SUMMARY_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn child_summary_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in child_summary_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
