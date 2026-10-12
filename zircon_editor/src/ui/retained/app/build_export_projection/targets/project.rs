use std::path::{Path, PathBuf};

use crate::ui::workbench::project::project_root_path;
use zircon_runtime::asset::project::ProjectManifest;

pub(super) fn load_active_project_manifest(
    project_path: impl AsRef<Path>,
) -> Result<(PathBuf, ProjectManifest), String> {
    let project_root = project_root_path(project_path).map_err(|error| error.to_string())?;
    let manifest_path = project_root.join("zircon-project.toml");
    let manifest = ProjectManifest::load(&manifest_path)
        .map_err(|error| format!("desktop export panel needs a project manifest: {error}"))?;
    Ok((project_root, manifest))
}
