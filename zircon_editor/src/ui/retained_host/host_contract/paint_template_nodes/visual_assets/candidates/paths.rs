use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;

pub(super) fn editor_asset_root() -> PathBuf {
    runtime_asset_path_with_dev_asset_root(Path::new(""), editor_dev_asset_root())
}

pub(super) fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub(super) fn is_editor_dev_asset_root(path: &Path) -> bool {
    path == editor_dev_asset_root()
}

pub(super) fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(super) fn normalized_asset_relative_path(source: &str) -> PathBuf {
    let source = source.trim();
    let value = if source.contains('\\') {
        Cow::Owned(source.replace('\\', "/"))
    } else {
        Cow::Borrowed(source)
    };
    let value = value.strip_prefix("res://").unwrap_or(value.as_ref());
    let mut relative = PathBuf::new();
    for component in Path::new(value.trim_start_matches('/')).components() {
        match component {
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => {}
            Component::Normal(value)
                if relative.as_os_str().is_empty() && value == OsStr::new("assets") => {}
            Component::Normal(value) => relative.push(value),
        }
    }
    relative
}

#[cfg(test)]
#[path = "paths/borrowed_normalization_tests.rs"]
mod borrowed_normalization_tests;
