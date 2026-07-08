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

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn typed_error_structure_status_row_source(
) -> String {
    super::status_docs::status_doc_rows_for_structure()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn typed_error_structure_status_map_source(
) -> String {
    super::status_docs::status_doc_status_map_for_structure()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn typed_error_structure_date_map_source(
) -> String {
    super::status_docs::status_doc_date_map_for_structure()
}
