use std::fs;
use std::path::{Path, PathBuf};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ATLAS_CACHE_DIR: &str =
    "editor-sprite-atlases";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn atlas_manifest_candidates(
    source_path: &Path,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let Some(root) = source_path
        .ancestors()
        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "assets"))
    else {
        return candidates;
    };
    if let Some(parent) = root.parent() {
        let atlas_dir = parent.join(".zircon").join("cache").join(ATLAS_CACHE_DIR);
        if let Ok(entries) = fs::read_dir(atlas_dir) {
            candidates =
                collect_atlas_manifest_candidates(entries.flatten().map(|entry| entry.path()));
        }
    }
    candidates
}

fn collect_atlas_manifest_candidates(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut candidates = paths
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    candidates
}

#[cfg(test)]
#[path = "discovery/atlas_candidate_tests.rs"]
mod atlas_candidate_tests;
