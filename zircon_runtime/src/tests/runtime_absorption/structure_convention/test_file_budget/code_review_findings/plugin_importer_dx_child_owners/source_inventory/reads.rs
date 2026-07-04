use super::super::super::super::*;

pub(super) fn plugin_importer_dx_sources() -> Vec<(&'static str, String)> {
    super::paths::plugin_importer_dx_source_paths()
        .iter()
        .map(|path| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn plugin_importer_dx_review_guard_count() -> usize {
    plugin_importer_dx_sources()
        .iter()
        .map(|(_, source)| source.matches("#[test]").count())
        .sum()
}
