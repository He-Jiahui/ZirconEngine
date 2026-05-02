use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

pub trait UiRuntimeTreeRoutingExt {
    fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError>;
}

impl UiRuntimeTreeRoutingExt for UiTree {
    fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
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
