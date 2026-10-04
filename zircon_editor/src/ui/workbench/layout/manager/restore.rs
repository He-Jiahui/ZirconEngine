use crate::ui::workbench::project::ProjectEditorWorkspace;

use super::super::{LayoutManager, RestorePolicy, WorkbenchLayout};

impl LayoutManager {
    /// 选择已加载的布局值；视图实例恢复、持久化读写与规范化另由宿主组织。
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
                .unwrap_or_else(|| self.default_layout()),
            RestorePolicy::PresetThenProjectThenGlobal { preset } => preset
                .or_else(|| project_workspace.map(|workspace| workspace.workbench))
                .or(global_default)
                .unwrap_or_else(|| self.default_layout()),
        })
    }
}

#[cfg(test)]
#[path = "tests/restore.rs"]
mod tests;
