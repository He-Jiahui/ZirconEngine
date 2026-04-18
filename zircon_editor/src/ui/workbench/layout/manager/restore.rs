use crate::ProjectEditorWorkspace;

use super::super::{LayoutManager, RestorePolicy, WorkbenchLayout};

impl LayoutManager {
    pub fn restore_workspace(
        &self,
        policy: RestorePolicy,
        project_workspace: Option<ProjectEditorWorkspace>,
        global_default: Option<WorkbenchLayout>,
    ) -> Result<WorkbenchLayout, String> {
        Ok(match policy {
            RestorePolicy::ProjectThenGlobal => project_workspace
                .map(|workspace| workspace.workbench)
                .or(global_default)
                .unwrap_or_default(),
            RestorePolicy::PresetThenProjectThenGlobal { preset } => preset
                .or_else(|| project_workspace.map(|workspace| workspace.workbench))
                .or(global_default)
                .unwrap_or_default(),
        })
    }
}
