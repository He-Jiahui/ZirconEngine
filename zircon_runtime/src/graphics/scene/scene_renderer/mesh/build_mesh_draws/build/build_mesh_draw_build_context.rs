use std::collections::HashSet;

use crate::graphics::types::ViewportRenderFrame;

use super::mesh_draw_build_context::MeshDrawBuildContext;

pub(super) fn build_mesh_draw_build_context(
    frame: &ViewportRenderFrame,
    _virtual_geometry_enabled: bool,
) -> MeshDrawBuildContext {
    let selection = frame
        .scene
        .overlays
        .selection
        .iter()
        .map(|highlight| highlight.owner)
        .collect::<HashSet<_>>();

    MeshDrawBuildContext {
        selection,
        allowed_virtual_geometry_entities: None,
    }
}
