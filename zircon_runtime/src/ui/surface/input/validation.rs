use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::super::surface::UiSurface;

pub(crate) fn require_valid_input_owner(
    surface: &UiSurface,
    node_id: UiNodeId,
) -> Result<(), String> {
    is_valid_input_owner(surface, node_id)
        .then_some(())
        .ok_or_else(|| format!("invalid input owner {node_id:?}"))
}

pub(crate) fn is_valid_input_owner(surface: &UiSurface, node_id: UiNodeId) -> bool {
    let Some(owner) = surface.tree.nodes.get(&node_id) else {
        return false;
    };
    if !owner.state_flags.enabled {
        return false;
    }

    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node) = surface.tree.nodes.get(&id) else {
            return false;
        };
        if !node.is_render_visible() {
            return false;
        }
        current = node.parent;
    }
    true
}
