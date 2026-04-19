use std::path::{Path, PathBuf};

pub(super) fn meta_path_for_source(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    path.with_file_name(format!("{file_name}.meta.toml"))
}
