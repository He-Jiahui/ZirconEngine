use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
};

use crate::ui::surface::ui_surface_effective_disabled;
use crate::ui::tree::UiRuntimeTreeAccessExt;

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
}
