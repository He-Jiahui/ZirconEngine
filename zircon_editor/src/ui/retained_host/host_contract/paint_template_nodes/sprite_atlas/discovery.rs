use std::fs;
use std::path::{Path, PathBuf};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ATLAS_LIBRARY_DIR:
    &str = "editor-sprite-atlases";

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
    push_candidate(
        &mut candidates,
        root.join("library").join(ATLAS_LIBRARY_DIR),
    );
    if let Some(parent) = root.parent() {
        push_candidate(
            &mut candidates,
            parent.join("library").join(ATLAS_LIBRARY_DIR),
        );
    }
    candidates
}

fn push_candidate(candidates: &mut Vec<PathBuf>, atlas_dir: PathBuf) {
    let Ok(entries) = fs::read_dir(atlas_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
            && !candidates.iter().any(|candidate| candidate == &path)
        {
            candidates.push(path);
        }
    }
    candidates.sort();
}
