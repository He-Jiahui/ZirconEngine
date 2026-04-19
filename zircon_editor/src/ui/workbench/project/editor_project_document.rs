use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::scene::Scene;

use super::project_editor_workspace::ProjectEditorWorkspace;

#[derive(Clone, Debug, PartialEq)]
pub struct EditorProjectDocument {
    pub root_path: PathBuf,
    pub manifest: ProjectManifest,
    pub world: Scene,
    pub editor_workspace: Option<ProjectEditorWorkspace>,
}
