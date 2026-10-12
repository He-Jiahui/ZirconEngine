use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::ProjectManifest;

pub(super) struct ModulePluginProjectManifestContext {
    pub(super) project_root: PathBuf,
    manifest_path: PathBuf,
    pub(super) manifest: ProjectManifest,
}

impl ModulePluginProjectManifestContext {
    pub(super) fn save(&self) -> Result<(), String> {
        self.manifest
            .save(&self.manifest_path)
            .map_err(|error| error.to_string())
    }
}

pub(super) fn load_module_plugin_project_manifest(
    project_path: impl AsRef<Path>,
) -> Result<ModulePluginProjectManifestContext, String> {
    let project_root = crate::ui::workbench::project::project_root_path(project_path)
        .map_err(|error| error.to_string())?;
    let manifest_path = project_root.join("zircon-project.toml");
    let manifest = ProjectManifest::load(&manifest_path).map_err(|error| error.to_string())?;
    Ok(ModulePluginProjectManifestContext {
        project_root,
        manifest_path,
        manifest,
    })
}
