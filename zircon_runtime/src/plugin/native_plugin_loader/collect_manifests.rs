use std::fs;
use std::path::{Path, PathBuf};

use super::PLUGIN_MANIFEST_FILE;

pub(super) fn collect_plugin_manifests(
    root: &Path,
    manifest_paths: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| {
        format!(
            "failed to enumerate native plugin root {}: {error}",
            root.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "failed to inspect native plugin entry under {}: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_plugin_manifests(&path, manifest_paths)?;
        } else if path.file_name().and_then(|value| value.to_str()) == Some(PLUGIN_MANIFEST_FILE) {
            manifest_paths.push(path);
        }
    }
    Ok(())
}
