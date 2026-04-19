use super::{UiInputPolicy, UiTree, UiTreeError};
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiPoint;

impl UiTree {
    pub(crate) fn effective_input_policy(
        &self,
        node_id: UiNodeId,
    ) -> Result<UiInputPolicy, UiTreeError> {
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

    pub(crate) fn supports_pointer(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .nodes
            .get(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(node.state_flags.enabled
            && (node.state_flags.clickable
                || node.state_flags.hoverable
                || node.state_flags.focusable))
    }

    pub(crate) fn first_scrollable_in_candidates(
        &self,
        candidates: &[UiNodeId],
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        for node_id in candidates {
            let node = self
                .nodes
                .get(node_id)
                .ok_or(UiTreeError::MissingNode(*node_id))?;
            if node.container.is_scrollable()
                && node.state_flags.visible
                && node.state_flags.enabled
            {
                return Ok(Some(*node_id));
            }
        }
        Ok(None)
    }

    pub(crate) fn passes_clip_chain(
        &self,
        node_id: UiNodeId,
        point: UiPoint,
    ) -> Result<bool, UiTreeError> {
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
