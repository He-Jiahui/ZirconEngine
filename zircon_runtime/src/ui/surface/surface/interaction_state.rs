use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::UiComponentEventReport,
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
};

use crate::ui::surface::ui_surface_effective_disabled;

use super::UiSurface;

impl UiSurface {
    pub(super) fn node_interaction_enabled(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(
            self.node_interaction_enabled_from_parts(
                node_id,
                node,
                node.template_metadata.as_ref(),
            ),
        )
    }

    pub(super) fn widget_interaction_enabled(
        &self,
        node_id: UiNodeId,
        node: &UiTreeNode,
        metadata: &UiTemplateNodeMetadata,
    ) -> bool {
        self.node_interaction_enabled_from_parts(node_id, node, Some(metadata))
    }

    fn node_interaction_enabled_from_parts(
        &self,
        node_id: UiNodeId,
        node: &UiTreeNode,
        metadata: Option<&UiTemplateNodeMetadata>,
    ) -> bool {
        !ui_surface_effective_disabled(self, node_id, node, metadata)
    }

    pub(crate) fn clear_hovered_input_path(
        &mut self,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        let hovered = std::mem::take(&mut self.focus.hovered);
        let mut reports = Vec::new();
        let mut changed_node_ids = BTreeSet::new();
        for node_id in &hovered {
            if self.component_states.set_hovered(*node_id, false) {
                changed_node_ids.insert(*node_id);
            }
        }
        self.mark_component_states_render_dirty(&changed_node_ids)?;
        for node_id in hovered {
            self.push_hover_leave_reports(node_id, &mut reports)?;
        }
        Ok(reports)
    }

    pub(crate) fn clear_pointer_interaction_without_route(
        &mut self,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        if let Some(pressed) = self.focus.pressed.take() {
            if self.component_states.set_pressed(pressed, false) {
                self.mark_component_state_render_dirty(pressed)?;
            }
        }
        self.release_pointer_capture();
        self.input.clear_last_cursor_point();
        self.clear_hovered_input_path()
    }

    fn push_hover_leave_reports(
        &self,
        node_id: UiNodeId,
        reports: &mut Vec<UiComponentEventReport>,
    ) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(());
        };
        for _ in metadata
            .bindings
            .iter()
            .filter(|binding| binding.event == UiEventKind::Hover)
        {
            reports.push(UiComponentEventReport {
                target: node_id,
                event: UiComponentEvent::Hover { hovered: false },
                delivered: true,
                drag: None,
                template_action: None,
            });
        }
        Ok(())
    }
}
