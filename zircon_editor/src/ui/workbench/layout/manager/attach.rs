use crate::ui::workbench::view::ViewHost;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    DocumentNode, LayoutCommandError, LayoutManager, MainHostPageLayout, TabInsertionAnchor,
    WorkbenchLayout,
};

impl LayoutManager {
    pub(crate) fn attach_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: ViewInstanceId,
        target: ViewHost,
        anchor: Option<TabInsertionAnchor>,
    ) -> Result<(), LayoutCommandError> {
        match target {
            ViewHost::Drawer(slot) => {
                let slot = slot.canonical();
                let drawer = layout
                    .active_activity_window_mut()
                    .and_then(|window| window.activity_drawers.get_mut(&slot))
                    .ok_or(LayoutCommandError::MissingDrawer { slot })?;
                drawer
                    .tab_stack
                    .insert(instance_id.clone(), anchor.as_ref());
                drawer.active_view = Some(instance_id);
                if drawer.mode == super::super::ActivityDrawerMode::Collapsed {
                    drawer.mode = super::super::ActivityDrawerMode::Pinned;
                }
            }
            ViewHost::Document(page_id, path) => {
                let node = self
                    .document_node_mut(layout, &page_id, &path)
                    .ok_or_else(|| LayoutCommandError::MissingDocumentNode {
                        page_id: page_id.clone(),
                        path: path.clone(),
                    })?;
                match node {
                    DocumentNode::Tabs(stack) => stack.insert(instance_id, anchor.as_ref()),
                    DocumentNode::SplitNode { .. } => {
                        return Err(LayoutCommandError::DocumentSplitAttachTarget {
                            page_id,
                            path,
                        });
                    }
                }
            }
            ViewHost::FloatingWindow(window_id, path) => {
                let window = layout
                    .floating_windows
                    .iter_mut()
                    .find(|window| window.window_id == window_id)
                    .ok_or_else(|| LayoutCommandError::MissingFloatingWindow {
                        window_id: window_id.clone(),
                    })?;
                let node = window.workspace.node_at_path_mut(&path).ok_or_else(|| {
                    LayoutCommandError::MissingFloatingWindowNode {
                        window_id: window_id.clone(),
                        path: path.clone(),
                    }
                })?;
                match node {
                    DocumentNode::Tabs(stack) => {
                        stack.insert(instance_id.clone(), anchor.as_ref());
                        window.focused_view = Some(instance_id);
                    }
                    DocumentNode::SplitNode { .. } => {
                        return Err(LayoutCommandError::FloatingSplitAttachTarget {
                            window_id,
                            path,
                        });
                    }
                }
            }
            ViewHost::ExclusivePage(page_id) => {
                layout
                    .main_pages
                    .push(MainHostPageLayout::ExclusiveActivityWindowPage {
                        id: page_id.clone(),
                        title: page_id.0.clone(),
                        window_instance: instance_id,
                    });
                layout.active_main_page = page_id;
            }
        }

        Ok(())
    }
}
