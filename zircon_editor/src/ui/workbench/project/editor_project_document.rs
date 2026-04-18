use std::path::PathBuf;

use zircon_asset::ProjectManifest;
use zircon_scene::Scene;

use super::project_editor_workspace::ProjectEditorWorkspace;

#[derive(Clone, Debug, PartialEq)]
pub struct EditorProjectDocument {
    pub root_path: PathBuf,
    pub manifest: ProjectManifest,
    pub world: Scene,
    pub editor_workspace: Option<ProjectEditorWorkspace>,
}
