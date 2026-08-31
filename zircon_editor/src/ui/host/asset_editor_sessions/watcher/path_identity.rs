use std::path::{Path, PathBuf};

pub(super) fn asset_id_for_watched_path(asset_roots: &[PathBuf], path: &Path) -> Option<String> {
    if !path.file_name()?.as_encoded_bytes().ends_with(b".zui") {
        return None;
    }
    let mut matching_roots = asset_roots.iter().filter(|root| path.starts_with(root));
    let asset_root = matching_roots.next()?;
    if matching_roots.next().is_some() {
        return None;
    }
    let relative = path.strip_prefix(asset_root).ok()?;
    let relative = String::from_utf8_lossy(relative.as_os_str().as_encoded_bytes());
    let mut asset_id = String::with_capacity("res://".len() + relative.len());
    asset_id.push_str("res://");
    asset_id.extend(
        relative
            .chars()
            .map(|character| if character == '\\' { '/' } else { character }),
    );
    Some(asset_id)
}

#[cfg(test)]
#[path = "path_identity/direct_join_tests.rs"]
mod direct_join_tests;
