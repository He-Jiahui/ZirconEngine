use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::super::super::surface::UiSurface;
use super::super::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult};

pub(super) fn require_node(
    surface: &UiSurface,
    node_id: UiNodeId,
) -> UiSurfaceInputEffectResult<()> {
    surface
        .tree
        .nodes
        .contains_key(&node_id)
        .then_some(())
        .ok_or(UiSurfaceInputEffectError::MissingNode { node_id })
}
