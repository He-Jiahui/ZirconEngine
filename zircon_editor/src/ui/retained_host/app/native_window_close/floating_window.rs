use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::{
    layout::{DocumentNode, LayoutCommand, MainPageId},
    view::ViewInstanceId,
};

use super::super::{callback_dispatch, RetainedEditorHost};

impl RetainedEditorHost {
    pub(super) fn close_floating_window_without_prompt(
        &mut self,
        window_id: &MainPageId,
        instance_ids: Vec<ViewInstanceId>,
    ) -> CloseRequestResponse {
        for instance_id in instance_ids {
            match callback_dispatch::dispatch_layout_command(
                &self.runtime,
                LayoutCommand::CloseView { instance_id },
            ) {
                Ok(effects) => self.apply_dispatch_effects(effects),
                Err(error) => {
                    self.set_status_line(error);
                    return CloseRequestResponse::KeepWindowShown;
                }
            }
        }

        self.recompute_if_dirty();
        let window_still_exists = self
            .runtime
            .current_layout()
            .floating_windows
            .iter()
            .any(|window| &window.window_id == window_id);
        if window_still_exists {
            CloseRequestResponse::KeepWindowShown
        } else {
            CloseRequestResponse::HideWindow
        }
    }

    pub(super) fn floating_window_close_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Option<Vec<ViewInstanceId>> {
        let layout = self.runtime.current_layout();
        let window = layout
            .floating_windows
            .iter()
            .find(|window| &window.window_id == window_id)?;
        let mut instances = Vec::new();
        collect_document_node_instances(&window.workspace, &mut instances);
        (!instances.is_empty()).then_some(instances)
    }
}

fn collect_document_node_instances(node: &DocumentNode, out: &mut Vec<ViewInstanceId>) {
    match node {
        DocumentNode::Tabs(stack) => out.extend(stack.tabs.iter().cloned()),
        DocumentNode::SplitNode { first, second, .. } => {
            collect_document_node_instances(first, out);
            collect_document_node_instances(second, out);
        }
    }
}
