use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::super::super::surface::UiSurface;

pub(super) fn require_node(surface: &UiSurface, node_id: UiNodeId) -> Result<(), String> {
    surface
        .tree
        .nodes
        .contains_key(&node_id)
        .then_some(())
        .ok_or_else(|| format!("missing node {node_id:?}"))
}
