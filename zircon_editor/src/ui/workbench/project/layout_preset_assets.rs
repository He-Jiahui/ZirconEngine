use std::fs;
use std::io;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime_interface::resource::ResourceScheme;

use super::constants::EDITOR_LAYOUT_PRESET_SUFFIX;
use super::layout_preset_asset_document::{
    decode_layout_preset_asset_document, encode_layout_preset_asset_document,
};
use super::layout_preset_asset_path::layout_preset_asset_path;
use crate::ui::workbench::layout::WorkbenchLayout;

pub(crate) fn save_layout_preset_asset(
    project: &ProjectManager,
    name: &str,
    layout: &WorkbenchLayout,
) -> Result<PathBuf, SceneProjectError> {
    let path = layout_preset_asset_path(project, name)?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let encoded = encode_layout_preset_asset_document(layout).map_err(io::Error::other)?;
    atomic_write(&path, encoded.as_bytes())?;
    Ok(path)
}

pub(crate) fn load_layout_preset_asset(
    project: &ProjectManager,
    name: &str,
) -> Result<Option<WorkbenchLayout>, SceneProjectError> {
    let path = layout_preset_asset_path(project, name)?;
    if !path.exists() {
        return Ok(None);
    }
    let source = fs::read(path)?;
    let workbench = decode_layout_preset_asset_document(&source)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Some(workbench))
}

pub(crate) fn list_layout_preset_assets(
    locators: impl IntoIterator<Item = AssetUri>,
) -> Vec<String> {
    let mut preset_names = locators
        .into_iter()
        .filter_map(|locator| layout_preset_name(&locator))
        .collect::<Vec<_>>();
    preset_names.sort_unstable();
    preset_names.dedup();
    preset_names
}

fn layout_preset_name(locator: &AssetUri) -> Option<String> {
    if locator.scheme() != ResourceScheme::Res || locator.label().is_some() {
        return None;
    }
    let relative = locator
        .path()
        .strip_prefix(&format!("{}/", super::constants::EDITOR_LAYOUT_PRESET_DIR))?;
    if relative.contains('/') {
        return None;
    }
    relative
        .strip_suffix(EDITOR_LAYOUT_PRESET_SUFFIX)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
#[path = "layout_preset_assets/optimization_tests.rs"]
mod optimization_tests;
