use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{AssetImportError, AssetUri};
use zircon_runtime::scene::world::SceneProjectError;

use super::constants::{EDITOR_LAYOUT_PRESET_DIR, EDITOR_LAYOUT_PRESET_SUFFIX};

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
#[path = "layout_preset_asset_path/single_buffer_sanitizer_tests.rs"]
mod single_buffer_sanitizer_tests;
