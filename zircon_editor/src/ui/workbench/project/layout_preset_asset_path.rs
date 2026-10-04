use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{AssetImportError, AssetUri};
use zircon_runtime::scene::world::SceneProjectError;

use super::constants::{EDITOR_LAYOUT_PRESET_DIR, EDITOR_LAYOUT_PRESET_SUFFIX};

/// 通过项目source owner定位res资源；规范文件名可能与输入显示名不同。
pub(in crate::ui::workbench::project) fn layout_preset_asset_path(
    project: &ProjectManager,
    name: &str,
) -> Result<PathBuf, SceneProjectError> {
    let relative = format!(
        "{}/{}{}",
        EDITOR_LAYOUT_PRESET_DIR,
        sanitize_layout_preset_name(name),
        EDITOR_LAYOUT_PRESET_SUFFIX
    );
    let uri = AssetUri::parse(&format!("res://{relative}")).map_err(AssetImportError::from)?;
    Ok(project.existing_or_primary_project_source_path_for_uri(&uri)?)
}

/// 维持历史文件名规则；不同原名可能归并到同一规范名，不能据原名认定文件唯一。
fn sanitize_layout_preset_name(name: &str) -> String {
    let mut sanitized = String::with_capacity(name.len());
    let mut pending_hyphens = 0_usize;
    for ch in name.chars() {
        let ch = match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        };
        if ch == '-' {
            if !sanitized.is_empty() {
                pending_hyphens += 1;
            }
            continue;
        }
        while pending_hyphens != 0 {
            sanitized.push('-');
            pending_hyphens -= 1;
        }
        sanitized.push(ch);
    }
    if sanitized.is_empty() {
        "preset".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
#[path = "layout_preset_asset_path/tests/single_buffer_sanitizer_tests.rs"]
mod single_buffer_sanitizer_tests;
