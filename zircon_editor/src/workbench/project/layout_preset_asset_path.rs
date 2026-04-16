use std::path::{Path, PathBuf};

use zircon_asset::ProjectPaths;
use zircon_scene::SceneProjectError;

use super::constants::{EDITOR_LAYOUT_PRESET_DIR, EDITOR_LAYOUT_PRESET_SUFFIX};

pub(in crate::workbench::project) fn layout_preset_asset_path(
    root: &Path,
    name: &str,
) -> Result<PathBuf, SceneProjectError> {
    let paths = ProjectPaths::from_root(root)?;
    Ok(paths
        .assets_root()
        .join(EDITOR_LAYOUT_PRESET_DIR)
        .join(format!(
            "{}{}",
            sanitize_layout_preset_name(name),
            EDITOR_LAYOUT_PRESET_SUFFIX
        )))
}

fn sanitize_layout_preset_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "preset".to_string()
    } else {
        sanitized
    }
}
