use super::*;

pub(in super::super) fn read_runtime_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(in super::super) fn read_runtime_absorption_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}
