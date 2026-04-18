use std::path::Path;

pub(super) fn preview_source_mtime(source_path: &Path) -> u64 {
    std::fs::metadata(source_path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
