use super::super::{repo_path as parent_repo_path, runtime_src_path as parent_runtime_src_path};

pub(super) const LOCK_UNWRAP_CALL: &str = concat!(".lock()", ".unwrap()");
pub(super) const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");

pub(super) fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    super::super::assert_contains_all(label, source, required);
}

pub(super) fn assert_contains_all_exact(label: &str, source: &str, required: &[&str]) {
    super::super::assert_contains_all_exact(label, source, required);
}

pub(super) fn assert_no_direct_lock_unwrap_in_production(label: &str, source: &str) {
    let production = production_section(source);
    assert!(
        !production.contains(LOCK_UNWRAP_CALL),
        "{label} production code should use poison-safe lock helpers instead of {LOCK_UNWRAP_CALL}"
    );
}

pub(super) fn production_section(source: &str) -> &str {
    source.split("\n#[cfg(test)]").next().unwrap_or(source)
}

pub(super) fn runtime_src_path(relative: &str) -> std::path::PathBuf {
    parent_runtime_src_path(relative)
}

pub(super) fn repo_path(relative: &str) -> std::path::PathBuf {
    parent_repo_path(relative)
}

pub(super) fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

pub(super) fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
