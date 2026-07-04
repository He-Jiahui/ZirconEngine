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
    let mut source = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        source.push_str(&child_source);
        source.push('\n');
    }
    source
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn p0_native_fixture_status_row_source(
) -> String {
    format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_ROWS),
        read_runtime_src(STRUCTURE_GUARD_ROW_PARENT),
        read_runtime_src(STRUCTURE_GUARD_ROWS),
    )
}
