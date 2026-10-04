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
    visit_valid_input_owner_path(surface, node_id, |_| {})
}

const INLINE_INPUT_OWNER_ROUTE_DEPTH: usize = 64;

pub(super) fn valid_input_owner_route(
    surface: &UiSurface,
    node_id: UiNodeId,
) -> Option<Vec<UiNodeId>> {
    let mut inline_route = [node_id; INLINE_INPUT_OWNER_ROUTE_DEPTH];
    let mut inline_len = 0;
    let mut overflow_route: Option<Vec<UiNodeId>> = None;
    let valid = visit_valid_input_owner_path(surface, node_id, |id| {
        if inline_len < INLINE_INPUT_OWNER_ROUTE_DEPTH {
            inline_route[inline_len] = id;
            inline_len += 1;
        } else {
            overflow_route
                .get_or_insert_with(|| {
                    let mut route = Vec::with_capacity(INLINE_INPUT_OWNER_ROUTE_DEPTH * 2);
                    route.extend_from_slice(&inline_route);
                    route
                })
                .push(id);
        }
    });
    if !valid {
        return None;
    }
    Some(overflow_route.unwrap_or_else(|| inline_route[..inline_len].to_vec()))
}

fn visit_valid_input_owner_path(
    surface: &UiSurface,
    node_id: UiNodeId,
    mut visit: impl FnMut(UiNodeId),
) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node) = surface.tree.nodes.get(&id) else {
            return false;
        };
        if !input_owner_node_is_valid(surface, id, node) {
            return false;
        }
        visit(id);
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
#[path = "validation/tests/visibility_first_tests.rs"]
mod visibility_first_tests;
