use std::path::{Path, PathBuf};

/// Only editor-owned, versioned Play snapshots receive a separate config writer.
pub(super) fn snapshot_config_path(relative: &str, snapshot: &Path) -> Option<PathBuf> {
    (relative.starts_with(".zircon/play/") && relative.ends_with(".zrscene.json"))
        .then(|| snapshot.with_extension("config.json"))
}
