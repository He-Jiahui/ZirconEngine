use std::path::{Path, PathBuf};

pub(super) fn asset_id_for_watched_path(asset_roots: &[PathBuf], path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_string_lossy();
    if !file_name.ends_with(".zui") {
        return None;
    }
    let mut matching_roots = asset_roots.iter().filter(|root| path.starts_with(root));
    let asset_root = matching_roots.next()?;
    if matching_roots.next().is_some() {
        return None;
    }
    let relative = path.strip_prefix(asset_root).ok()?;
    let normalized = relative.to_string_lossy().replace('\\', "/");
    Some(format!("res://{normalized}"))
}
