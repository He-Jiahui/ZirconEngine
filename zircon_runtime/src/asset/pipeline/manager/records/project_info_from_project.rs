use super::ProjectInfo;
use crate::asset::project::ProjectManager;
use crate::core::resource::ResourceState;

impl ProjectInfo {
    /// Captures one already-activated project's manifest and registry state without rescanning it.
    pub fn from_project(project: &ProjectManager) -> Self {
        let mut asset_count = 0;
        let mut ready_asset_count = 0;
        let mut failed_asset_count = 0;
        for record in project.registry().values() {
            asset_count += 1;
            match record.state {
                ResourceState::Ready => ready_asset_count += 1,
                ResourceState::Error => failed_asset_count += 1,
                ResourceState::Pending | ResourceState::Reloading => {}
            }
        }
        Self {
            root_path: project.paths().root().to_string_lossy().into_owned(),
            name: project.manifest().name.clone(),
            default_scene_uri: project.manifest().default_scene.to_string(),
            library_version: project.manifest().library_version,
            asset_count,
            ready_asset_count,
            failed_asset_count,
            registry_diagnostic_count: project.asset_registry().diagnostics().len(),
        }
    }
}

pub(in crate::asset::pipeline::manager) fn build_project_info(
    project: &ProjectManager,
) -> ProjectInfo {
    ProjectInfo::from_project(project)
}
