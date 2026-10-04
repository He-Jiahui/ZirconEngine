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

// TODO: [CR-EDITOR-WORKBENCH-0009] 确认保存前是否按规范文件名处理名称碰撞；不同输入名可解析到同一路径并覆盖，当前宿主直接传原名。
/// 保存项目preset资源；返回source路径后宿主还须导入目录，使发现列表进入新代次。
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

/// 项目资源缺失返回None供全局配置回退；文件无效保持错误，不伪装成缺失。
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

/// 从当前source目录定位发现直接preset资源，按规范文件名排序去重。
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
#[path = "layout_preset_assets/tests/optimization_tests.rs"]
mod optimization_tests;
