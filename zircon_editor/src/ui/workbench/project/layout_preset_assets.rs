use std::fs;
use std::path::PathBuf;

use zircon_runtime::asset::AssetUri;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime_interface::resource::ResourceScheme;

use super::constants::{EDITOR_LAYOUT_PRESET_FORMAT_VERSION, EDITOR_LAYOUT_PRESET_SUFFIX};
use super::layout_preset_asset_document::LayoutPresetAssetDocument;
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
    let document = LayoutPresetAssetDocument {
        format_version: EDITOR_LAYOUT_PRESET_FORMAT_VERSION,
        preset_name: name.to_string(),
        workbench: layout.clone(),
    };
    fs::write(&path, serde_json::to_string_pretty(&document)?)?;
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
    let document = serde_json::from_str::<LayoutPresetAssetDocument>(&fs::read_to_string(path)?)?;
    Ok(Some(document.workbench))
}

pub(crate) fn list_layout_preset_assets(
    locators: impl IntoIterator<Item = AssetUri>,
) -> Vec<String> {
    let mut preset_names = locators
        .into_iter()
        .filter_map(|locator| layout_preset_name(&locator))
        .collect::<Vec<_>>();
    preset_names.sort();
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
