use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::super::surface::UiSurface;
use super::super::ui_surface_node_disabled;
use super::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult};

pub(crate) fn require_valid_input_owner(
    surface: &UiSurface,
    node_id: UiNodeId,
) -> UiSurfaceInputEffectResult<()> {
    is_valid_input_owner(surface, node_id)
        .then_some(())
        .ok_or(UiSurfaceInputEffectError::InvalidInputOwner { node_id })
}

pub(crate) fn is_valid_input_owner(surface: &UiSurface, node_id: UiNodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node) = surface.tree.nodes.get(&id) else {
            return false;
        };
        if !input_owner_node_is_valid(surface, id, node) {
            return false;
        }
        current = node.parent;
    }
    true
}

fn input_owner_node_is_valid(
    surface: &UiSurface,
    node_id: UiNodeId,
    node: &zircon_runtime_interface::ui::tree::UiTreeNode,
) -> bool {
    node.is_render_visible()
        && !ui_surface_node_disabled(surface, node_id, node, node.template_metadata.as_ref())
}

#[cfg(test)]
#[path = "validation/visibility_first_tests.rs"]
mod visibility_first_tests;
