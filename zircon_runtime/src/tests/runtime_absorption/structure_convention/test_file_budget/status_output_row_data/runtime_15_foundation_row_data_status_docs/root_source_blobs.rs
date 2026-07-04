use super::*;

pub(super) fn runtime_15_row_count(source: &str) -> usize {
    source
        .lines()
        .filter(|line| line.starts_with("        \"Runtime 15 "))
        .count()
}

pub(super) fn status_doc_child_sources() -> Vec<(&'static str, String)> {
    STATUS_DOC_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in status_doc_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
