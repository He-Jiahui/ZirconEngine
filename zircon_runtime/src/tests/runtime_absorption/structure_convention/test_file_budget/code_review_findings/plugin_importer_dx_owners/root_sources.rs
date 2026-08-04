use super::*;

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn folder_backed_child_sources(
) -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn folder_backed_child_source_blob(
) -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}
