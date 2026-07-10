use std::fs;
use std::path::Path;

#[path = "support/status_evidence.rs"]
mod status_evidence;

pub(super) use status_evidence::{
    read_runtime_15_naming_date_map, read_runtime_15_naming_status_map,
    read_runtime_15_naming_status_rows,
};

pub(super) fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    let missing = required
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} missing required anchors: {missing:?}"
    );
}

pub(super) fn read_text(path: &Path, label: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{label}: {error}"))
}

pub(super) fn read_repo_text(manifest_root: &Path, relative_path: &str) -> String {
    let repo_root = manifest_root
        .parent()
        .expect("zircon_runtime manifest should live under repository root");
    read_text(
        &repo_root.join(relative_path),
        "repository document should be readable",
    )
}
