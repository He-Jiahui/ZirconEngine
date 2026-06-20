use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::plugin::PluginPackageManifest;

pub(super) fn find_native_package_dir(
    root: &Path,
    package_id: &str,
) -> Result<Option<PathBuf>, std::io::Error> {
    if !is_real_directory(root)? {
        return Ok(None);
    }

    if let Some(direct) = direct_child_package_dir(root, package_id) {
        if is_real_directory(&direct)?
            && package_manifest_matches(&direct.join("plugin.toml"), package_id)?
        {
            return Ok(Some(direct));
        }
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() || !file_type.is_dir() {
                continue;
            }
            let path = entry.path();
            if package_manifest_matches(&path.join("plugin.toml"), package_id)? {
                return Ok(Some(path));
            }
            stack.push(path);
        }
    }

    Ok(None)
}

fn is_real_directory(path: &Path) -> Result<bool, std::io::Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn direct_child_package_dir(root: &Path, package_id: &str) -> Option<PathBuf> {
    let mut components = Path::new(package_id).components();
    let Some(Component::Normal(_)) = components.next() else {
        return None;
    };
    if components.next().is_some() {
        return None;
    }
    Some(root.join(package_id))
}

fn package_manifest_matches(path: &Path, package_id: &str) -> Result<bool, std::io::Error> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(false);
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Ok(false);
    }

    let source = fs::read_to_string(path)?;
    Ok(toml::from_str::<PluginPackageManifest>(&source)
        .map(|manifest| manifest.id == package_id)
        .unwrap_or(false))
}
