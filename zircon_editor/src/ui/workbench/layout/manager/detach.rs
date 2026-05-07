use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    ActivityDrawerMode, DocumentNode, LayoutManager, MainHostPageLayout, WorkbenchLayout,
};

impl LayoutManager {
    pub(crate) fn detach_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: &ViewInstanceId,
    ) -> bool {
        let mut changed = false;

        for activity_window in layout.activity_windows.values_mut() {
            for drawer in activity_window.activity_drawers.values_mut() {
                changed |= drawer.tab_stack.remove(instance_id);
                if drawer.active_view.as_ref() == Some(instance_id) {
                    drawer.active_view = drawer.tab_stack.active_tab.clone();
                }
                if drawer.active_view.is_none() {
                    drawer.mode = ActivityDrawerMode::Collapsed;
                }
            }
        }

        for page in &mut layout.main_pages {
            if let Some(workspace) = page.document_workspace_mut() {
                changed |= workspace.remove_instance(instance_id);
            }
        }

        for window in &mut layout.floating_windows {
            changed |= window.workspace.remove_instance(instance_id);
            if window.focused_view.as_ref() == Some(instance_id) {
                window.focused_view = None;
            }
        }

        layout.main_pages.retain(|page| match page {
            MainHostPageLayout::WorkbenchPage { .. } => true,
            MainHostPageLayout::ExclusiveActivityWindowPage {
                window_instance, ..
            } => window_instance != instance_id,
        });
        layout
            .floating_windows
            .retain(|window| match &window.workspace {
                DocumentNode::Tabs(stack) => !stack.tabs.is_empty(),
                DocumentNode::SplitNode { .. } => true,
            });

        changed
    }
}
