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

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn p0_native_fixture_structure_guard_child_source_blob(
) -> String {
    let mut source = String::new();
    source.push_str(&read_runtime_src(STRUCTURE_GUARD_OWNER));
    source.push('\n');
    for (path, child_source) in folder_backed_child_sources() {
        source.push_str(path);
        source.push('\n');
        source.push_str(&child_source);
        source.push('\n');
    }
    source.push_str(&read_runtime_src(P0_NATIVE_FIXTURE_ROOT_STATUSES_CHILD));
    source.push('\n');
    for path in [PARENT, SDK_MACRO_LEAF, IMPORTER_LEAF] {
        source.push_str(path);
        source.push('\n');
        source.push_str(&read_runtime_src(path));
        source.push('\n');
    }
    source
}
