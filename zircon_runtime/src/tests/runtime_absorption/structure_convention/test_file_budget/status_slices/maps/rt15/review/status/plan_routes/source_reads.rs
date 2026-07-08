use super::*;

pub(super) fn read_sources(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|path| read_runtime_src(path)).collect()
}
