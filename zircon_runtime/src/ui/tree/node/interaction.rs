use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiPoint,
    tree::{UiInputPolicy, UiTree, UiTreeError},
};

pub trait UiRuntimeTreeInteractionExt {
    fn effective_input_policy(&self, node_id: UiNodeId) -> Result<UiInputPolicy, UiTreeError>;
    fn supports_pointer(&self, node_id: UiNodeId) -> Result<bool, UiTreeError>;
    fn first_scrollable_in_candidates(
        &self,
        candidates: &[UiNodeId],
    ) -> Result<Option<UiNodeId>, UiTreeError>;
    fn passes_clip_chain(&self, node_id: UiNodeId, point: UiPoint) -> Result<bool, UiTreeError>;
}

impl UiRuntimeTreeInteractionExt for UiTree {
    fn effective_input_policy(&self, node_id: UiNodeId) -> Result<UiInputPolicy, UiTreeError> {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            match node.input_policy {
                UiInputPolicy::Inherit => current = node.parent,
                explicit => return Ok(explicit),
            }
        }
        Ok(UiInputPolicy::Receive)
    }

    fn supports_pointer(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .nodes
            .get(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(node.supports_pointer())
    }

    fn first_scrollable_in_candidates(
        &self,
        candidates: &[UiNodeId],
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        for node_id in candidates {
            let node = self
                .nodes
                .get(node_id)
                .ok_or(UiTreeError::MissingNode(*node_id))?;
            if node.container.is_scrollable()
                && node.state_flags.enabled
                && node.is_render_visible()
            {
                return Ok(Some(*node_id));
            }
        }
        Ok(None)
    }

    fn passes_clip_chain(&self, node_id: UiNodeId, point: UiPoint) -> Result<bool, UiTreeError> {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            if node.clip_to_bounds {
                let clip_frame = node
                    .layout_cache
                    .clip_frame
                    .unwrap_or(node.layout_cache.frame);
                if !clip_frame.contains_point(point) {
                    return Ok(false);
                }
            }
            current = node.parent;
        }
        Ok(true)
    }
}
