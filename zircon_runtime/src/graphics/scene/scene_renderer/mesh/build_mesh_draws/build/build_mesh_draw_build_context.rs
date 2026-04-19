use std::collections::HashSet;

use crate::graphics::types::EditorOrRuntimeFrame;

use super::super::build_virtual_geometry_cluster_raster_draws::build_virtual_geometry_cluster_raster_draws;
use super::mesh_draw_build_context::MeshDrawBuildContext;

pub(super) fn build_mesh_draw_build_context(
    frame: &EditorOrRuntimeFrame,
    virtual_geometry_enabled: bool,
) -> MeshDrawBuildContext {
    let selection = frame
        .scene
        .overlays
        .selection
        .iter()
        .map(|highlight| highlight.owner)
        .collect::<HashSet<_>>();
    let allowed_virtual_geometry_entities = virtual_geometry_enabled.then(|| {
        frame
            .virtual_geometry_prepare
            .as_ref()
            .map(|prepare| {
                prepare
                    .visible_entities
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default()
    });
    let virtual_geometry_cluster_draws =
        virtual_geometry_enabled.then(|| build_virtual_geometry_cluster_raster_draws(frame));

    MeshDrawBuildContext {
        selection,
        virtual_geometry_enabled,
        allowed_virtual_geometry_entities,
        virtual_geometry_cluster_draws,
    }
}
