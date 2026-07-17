use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    ActivityDrawerMode, DocumentNode, LayoutManager, MainHostPageLayout, WorkbenchLayout,
};

impl LayoutManager {
    pub(crate) fn focus_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: &ViewInstanceId,
    ) -> bool {
        if layout.activity_windows.is_empty() {
            layout.default_activity_window_mut();
        }
        let mut focused_activity_window = None;
        for (activity_window_id, activity_window) in layout.activity_windows.iter_mut() {
            for drawer in activity_window.activity_drawers.values_mut() {
                if drawer.tab_stack.tabs.contains(instance_id) {
                    let changed = drawer.tab_stack.active_tab.as_ref() != Some(instance_id)
                        || drawer.active_view.as_ref() != Some(instance_id)
                        || drawer.mode == ActivityDrawerMode::Collapsed;
                    drawer.tab_stack.active_tab = Some(instance_id.clone());
                    drawer.active_view = Some(instance_id.clone());
                    if drawer.mode == ActivityDrawerMode::Collapsed {
                        drawer.mode = ActivityDrawerMode::Pinned;
                    }
                    focused_activity_window = Some((activity_window_id.clone(), changed));
                    break;
                }
            }
            if focused_activity_window.is_some() {
                break;
            }
        }
        if let Some((activity_window_id, mut changed)) = focused_activity_window {
            if let Some(page_id) = layout.page_id_for_activity_window(&activity_window_id) {
                changed |= layout.active_main_page != page_id;
                if changed {
                    layout.active_main_page = page_id;
                }
            }
            return changed;
        }

        for page in &mut layout.main_pages {
            if let Some(workspace) = page.document_workspace_mut() {
                if let Some(mut changed) = Self::focus_in_document_node(workspace, instance_id) {
                    let page_id = page.id().clone();
                    changed |= layout.active_main_page != page_id;
                    if changed {
                        layout.active_main_page = page_id;
                    }
                    return changed;
                }
            } else if let MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } = page
            {
                if window_instance == instance_id {
                    let changed = layout.active_main_page != id.clone();
                    if changed {
                        layout.active_main_page = id.clone();
                    }
                    return changed;
                }
            }
        }

        for window in &mut layout.floating_windows {
            if window.workspace.contains(instance_id) {
                let changed = window.focused_view.as_ref() != Some(instance_id);
                if changed {
                    window.focused_view = Some(instance_id.clone());
                }
                return changed;
            }
        }

        false
    }

    fn focus_in_document_node(
        node: &mut DocumentNode,
        instance_id: &ViewInstanceId,
    ) -> Option<bool> {
        match node {
            DocumentNode::Tabs(stack) => {
                if stack.tabs.contains(instance_id) {
                    let changed = stack.active_tab.as_ref() != Some(instance_id);
                    if changed {
                        stack.active_tab = Some(instance_id.clone());
                    }
                    Some(changed)
                } else {
                    None
                }
            }
            DocumentNode::SplitNode { first, second, .. } => {
                Self::focus_in_document_node(first, instance_id)
                    .or_else(|| Self::focus_in_document_node(second, instance_id))
            }
        }
    }
}
