use crate::ui::workbench::project::ProjectEditorWorkspace;

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
                .unwrap_or_else(|| self.default_layout()),
            RestorePolicy::PresetThenProjectThenGlobal { preset } => preset
                .or_else(|| project_workspace.map(|workspace| workspace.workbench))
                .or(global_default)
                .unwrap_or_else(|| self.default_layout()),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::workbench::layout::{
        ActivityDrawerSlot, ActivityWindowId, DocumentNode, LayoutManager, MainHostPageLayout,
        RestorePolicy,
    };
    use crate::ui::workbench::view::ViewInstanceId;

    #[test]
    fn project_then_global_restore_falls_back_to_material_fyrox_workbench() {
        let manager = LayoutManager;
        let layout = manager
            .restore_workspace(RestorePolicy::ProjectThenGlobal, None, None)
            .unwrap();

        assert_default_material_fyrox_workbench(&layout);
    }

    #[test]
    fn preset_restore_falls_back_to_material_fyrox_workbench() {
        let manager = LayoutManager;
        let layout = manager
            .restore_workspace(
                RestorePolicy::PresetThenProjectThenGlobal { preset: None },
                None,
                None,
            )
            .unwrap();

        assert_default_material_fyrox_workbench(&layout);
    }

    fn assert_default_material_fyrox_workbench(layout: &super::WorkbenchLayout) {
        let MainHostPageLayout::WorkbenchPage { id, .. } = &layout.main_pages[0] else {
            panic!("restore fallback should open the default workbench page");
        };
        let document_workspace = layout
            .content_workspace_for_page(id)
            .expect("workbench page should resolve its activity-window content workspace");
        let DocumentNode::Tabs(documents) = document_workspace else {
            panic!("workbench document area should use the preset tab stack");
        };
        assert_eq!(
            documents.tabs,
            vec![
                ViewInstanceId::new("editor.scene#1"),
                ViewInstanceId::new("editor.game#1")
            ]
        );

        assert_eq!(
            layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftTop]
                .tab_stack
                .tabs,
            vec![
                ViewInstanceId::new("editor.hierarchy#1"),
                ViewInstanceId::new("editor.assets#1")
            ]
        );
        assert!(layout
            .activity_windows
            .contains_key(&ActivityWindowId::new("window:material_editor")));
        assert!(layout
            .activity_windows
            .contains_key(&ActivityWindowId::new("window:animation_editor")));
    }
}
