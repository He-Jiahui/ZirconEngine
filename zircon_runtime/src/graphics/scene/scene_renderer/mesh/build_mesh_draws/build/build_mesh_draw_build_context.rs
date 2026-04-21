use std::collections::HashSet;

use crate::graphics::types::ViewportRenderFrame;

use super::super::build_virtual_geometry_cluster_raster_draws::build_virtual_geometry_cluster_raster_draws;
use super::mesh_draw_build_context::MeshDrawBuildContext;

pub(super) fn build_mesh_draw_build_context(
    frame: &ViewportRenderFrame,
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
        if let Some(prepare) = frame.virtual_geometry_prepare.as_ref() {
            prepare
                .visible_entities
                .iter()
                .copied()
                .collect::<HashSet<_>>()
        } else {
            frame
                .virtual_geometry_cluster_selections
                .as_ref()
                .map(|selections| {
                    selections
                        .iter()
                        .map(|selection| selection.entity)
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default()
        }
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

#[cfg(test)]
mod tests {
    use super::build_mesh_draw_build_context;
    use crate::core::math::UVec2;
    use crate::graphics::types::{
        ViewportRenderFrame, VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
    };
    use crate::scene::world::World;
    use std::collections::HashSet;

    #[test]
    fn build_context_accepts_frame_owned_virtual_geometry_cluster_selections_without_prepare() {
        let world = World::new();
        let entity = world
            .nodes()
            .iter()
            .find(|node| node.mesh.is_some())
            .map(|node| node.id)
            .expect("default world should contain a renderable mesh");
        let frame =
            ViewportRenderFrame::from_extract(world.to_render_frame_extract(), UVec2::new(64, 64))
                .with_virtual_geometry_cluster_selections(Some(vec![
                    VirtualGeometryClusterSelection {
                        submission_index: 0,
                        instance_index: Some(0),
                        entity,
                        cluster_id: 400,
                        cluster_ordinal: 1,
                        page_id: 300,
                        lod_level: 3,
                        submission_page_id: 300,
                        submission_lod_level: 3,
                        entity_cluster_start_ordinal: 1,
                        entity_cluster_span_count: 1,
                        entity_cluster_total_count: 2,
                        lineage_depth: 0,
                        frontier_rank: 7,
                        resident_slot: Some(4),
                        submission_slot: Some(4),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                ]));

        let build_context = build_mesh_draw_build_context(&frame, true);

        assert_eq!(
            build_context.allowed_virtual_geometry_entities,
            Some(HashSet::from([entity])),
            "expected frame-owned VG cluster selection input to keep the entity eligible even when prepare is absent"
        );
        assert_eq!(
            build_context
                .virtual_geometry_cluster_draws
                .as_ref()
                .and_then(|draws| draws.get(&entity))
                .map(Vec::as_slice),
            Some(&[crate::graphics::types::VirtualGeometryClusterRasterDraw {
                submission_index: 0,
                instance_index: Some(0),
                page_id: 300,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 2,
                lineage_depth: 0,
                lod_level: 3,
                frontier_rank: 7,
                resident_slot: Some(4),
                submission_slot: Some(4),
                state: VirtualGeometryPrepareClusterState::Resident,
            }][..]),
            "expected mesh build context to project fallback raster draws from the precomputed runtime-frame VG cluster selection seam instead of requiring direct prepare access"
        );
    }
}
