use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::super::surface::UiSurface;
use super::super::ui_surface_node_disabled;

pub(crate) fn require_valid_input_owner(
    surface: &UiSurface,
    node_id: UiNodeId,
) -> Result<(), String> {
    is_valid_input_owner(surface, node_id)
        .then_some(())
        .ok_or_else(|| format!("invalid input owner {node_id:?}"))
}

pub(crate) fn is_valid_input_owner(surface: &UiSurface, node_id: UiNodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node) = surface.tree.nodes.get(&id) else {
            return false;
        };
        if ui_surface_node_disabled(surface, id, node, node.template_metadata.as_ref()) {
            return false;
        }
        if !node.is_render_visible() {
            return false;
        }
        current = node.parent;
    }
    true
}
