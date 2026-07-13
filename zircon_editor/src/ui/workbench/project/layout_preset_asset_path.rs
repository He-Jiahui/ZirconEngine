use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{AssetImportError, AssetUri};
use zircon_runtime::scene::world::SceneProjectError;

use super::constants::{EDITOR_LAYOUT_PRESET_DIR, EDITOR_LAYOUT_PRESET_SUFFIX};

pub(in crate::ui::workbench::project) fn layout_preset_asset_path(
    root: &Path,
    name: &str,
) -> Result<PathBuf, SceneProjectError> {
    let project = ProjectManager::open(root)?;
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
