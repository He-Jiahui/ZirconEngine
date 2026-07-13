use std::fs;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::world::SceneProjectError;

use super::constants::{EDITOR_LAYOUT_PRESET_FORMAT_VERSION, EDITOR_LAYOUT_PRESET_SUFFIX};
use super::layout_preset_asset_document::LayoutPresetAssetDocument;
use super::layout_preset_asset_path::layout_preset_asset_path;
use super::project_root_path::project_root_path;
use crate::ui::workbench::layout::WorkbenchLayout;

pub(crate) fn save_layout_preset_asset(
    root: impl AsRef<std::path::Path>,
    name: &str,
    layout: &WorkbenchLayout,
) -> Result<PathBuf, SceneProjectError> {
    let root = project_root_path(root)?;
    let path = layout_preset_asset_path(&root, name)?;
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
    root: impl AsRef<std::path::Path>,
    name: &str,
) -> Result<Option<WorkbenchLayout>, SceneProjectError> {
    let root = project_root_path(root)?;
    let path = layout_preset_asset_path(&root, name)?;
    if !path.exists() {
        return Ok(None);
    }
    let document = serde_json::from_str::<LayoutPresetAssetDocument>(&fs::read_to_string(path)?)?;
    Ok(Some(document.workbench))
}

pub(crate) fn list_layout_preset_assets(
    root: impl AsRef<std::path::Path>,
) -> Result<Vec<String>, SceneProjectError> {
    let root = project_root_path(root)?;
    let mut preset_names = Vec::new();
    let project = ProjectManager::open(&root)?;
    for asset_root in project.project_asset_roots() {
        let preset_dir = asset_root.join(super::constants::EDITOR_LAYOUT_PRESET_DIR);
        if !preset_dir.exists() {
            continue;
        }
        collect_layout_preset_names(&preset_dir, &mut preset_names)?;
    }
    preset_names.sort();
    preset_names.dedup();
    Ok(preset_names)
}

fn collect_layout_preset_names(
    preset_dir: &std::path::Path,
    preset_names: &mut Vec<String>,
) -> Result<(), SceneProjectError> {
    for entry in fs::read_dir(preset_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(EDITOR_LAYOUT_PRESET_SUFFIX))
        {
            continue;
        }
        let name = fs::read_to_string(&path)
            .ok()
            .and_then(|contents| serde_json::from_str::<LayoutPresetAssetDocument>(&contents).ok())
            .map(|document| document.preset_name)
            .or_else(|| {
                path.file_name()
                    .and_then(|value| value.to_str())
                    .map(|value| {
                        value
                            .trim_end_matches(EDITOR_LAYOUT_PRESET_SUFFIX)
                            .to_string()
                    })
            });
        if let Some(name) = name {
            preset_names.push(name);
        }
    }
    Ok(())
}
