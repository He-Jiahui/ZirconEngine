use crate::{ViewHost, ViewInstanceId};

use super::super::{
    DocumentNode, LayoutManager, MainHostPageLayout, TabInsertionAnchor, WorkbenchLayout,
};

impl LayoutManager {
    pub(crate) fn attach_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: ViewInstanceId,
        target: ViewHost,
        anchor: Option<TabInsertionAnchor>,
    ) -> Result<(), String> {
        match target {
            ViewHost::Drawer(slot) => {
                let drawer = layout
                    .drawers
                    .get_mut(&slot)
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                drawer
                    .tab_stack
                    .insert(instance_id.clone(), anchor.as_ref());
                drawer.active_view = Some(instance_id);
            }
            ViewHost::Document(page_id, path) => {
                let node = self
                    .document_node_mut(layout, &page_id, &path)
                    .ok_or_else(|| format!("missing document node on page {}", page_id.0))?;
                match node {
                    DocumentNode::Tabs(stack) => stack.insert(instance_id, anchor.as_ref()),
                    DocumentNode::SplitNode { .. } => {
                        return Err("cannot attach directly to split node".to_string())
                    }
                }
            }
            ViewHost::FloatingWindow(window_id, path) => {
                let window = layout
                    .floating_windows
                    .iter_mut()
                    .find(|window| window.window_id == window_id)
                    .ok_or_else(|| format!("missing floating window {}", window_id.0))?;
                let node = window
                    .workspace
                    .node_at_path_mut(&path)
                    .ok_or_else(|| format!("missing floating window node {}", window_id.0))?;
                match node {
                    DocumentNode::Tabs(stack) => {
                        stack.insert(instance_id.clone(), anchor.as_ref());
                        window.focused_view = Some(instance_id);
                    }
                    DocumentNode::SplitNode { .. } => {
                        return Err("cannot attach directly to split node".to_string())
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
