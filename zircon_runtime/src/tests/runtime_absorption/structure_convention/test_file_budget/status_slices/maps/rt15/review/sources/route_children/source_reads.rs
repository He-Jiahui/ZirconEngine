use super::*;

pub(in super::super::super) fn read_review_root_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}
