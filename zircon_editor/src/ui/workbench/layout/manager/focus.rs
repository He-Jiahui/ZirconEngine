use crate::ui::workbench::view::ViewInstanceId;

use super::super::{DocumentNode, LayoutManager, MainHostPageLayout, WorkbenchLayout};

impl LayoutManager {
    pub(crate) fn focus_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: &ViewInstanceId,
    ) -> bool {
        for activity_window in layout.activity_windows.values_mut() {
            for (slot, drawer) in &mut activity_window.activity_drawers {
                if drawer.tab_stack.tabs.contains(instance_id) {
                    drawer.tab_stack.active_tab = Some(instance_id.clone());
                    drawer.active_view = Some(instance_id.clone());
                    if let Some(root_drawer) = layout.drawers.get_mut(slot) {
                        root_drawer.tab_stack.active_tab = drawer.tab_stack.active_tab.clone();
                        root_drawer.active_view = drawer.active_view.clone();
                    }
                    return true;
                }
            }
        }

        for page in &mut layout.main_pages {
            if let Some(workspace) = page.document_workspace_mut() {
                if Self::focus_in_document_node(workspace, instance_id) {
                    layout.active_main_page = page.id().clone();
                    return true;
                }
            } else if let MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } = page
            {
                if window_instance == instance_id {
                    layout.active_main_page = id.clone();
                    return true;
                }
            }
        }

        for window in &mut layout.floating_windows {
            if window.workspace.contains(instance_id) {
                window.focused_view = Some(instance_id.clone());
                return true;
            }
        }

        false
    }

    fn focus_in_document_node(node: &mut DocumentNode, instance_id: &ViewInstanceId) -> bool {
        match node {
            DocumentNode::Tabs(stack) => {
                if stack.tabs.contains(instance_id) {
                    stack.active_tab = Some(instance_id.clone());
                    true
                } else {
                    false
                }
            }
            DocumentNode::SplitNode { first, second, .. } => {
                Self::focus_in_document_node(first, instance_id)
                    || Self::focus_in_document_node(second, instance_id)
            }
        }
    }
}
