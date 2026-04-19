use super::ProjectInfo;
use crate::project::ProjectManager;

pub(in crate::pipeline::manager) fn build_project_info(project: &ProjectManager) -> ProjectInfo {
    ProjectInfo {
        root_path: project.paths().root().to_string_lossy().into_owned(),
        name: project.manifest().name.clone(),
        default_scene_uri: project.manifest().default_scene.to_string(),
        library_version: project.manifest().library_version,
    }
}
