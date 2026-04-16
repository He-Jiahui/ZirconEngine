use super::{UiTree, UiTreeError};
use crate::UiNodeId;

impl UiTree {
    pub fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        let mut route = Vec::new();
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            route.push(id);
            current = node.parent;
        }
        Ok(route)
    }
}
