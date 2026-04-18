use crate::ProjectEditorWorkspace;

use super::super::{LayoutManager, WorkbenchLayout};

impl LayoutManager {
    pub fn load_global_default(&self, config: Option<WorkbenchLayout>) -> Option<WorkbenchLayout> {
        config
    }

    pub fn load_project_workspace(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Option<ProjectEditorWorkspace> {
        workspace
    }

    pub fn save_global_default(&self, layout: &WorkbenchLayout) -> WorkbenchLayout {
        layout.clone()
    }

    pub fn save_project_workspace(
        &self,
        workspace: &ProjectEditorWorkspace,
    ) -> ProjectEditorWorkspace {
        workspace.clone()
    }
}
