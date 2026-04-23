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
        if frame.has_explicit_virtual_geometry_cluster_selection_override() {
            return frame
                .virtual_geometry_cluster_selections
                .as_ref()
                .map(|selections| selection_entities(selections.as_slice()))
                .unwrap_or_default();
        }

        frame.virtual_geometry_prepare.as_ref().map_or_else(
            || {
                frame
                    .resolved_virtual_geometry_cluster_selections()
                    .map(|selections| selection_entities(selections.as_ref()))
                    .unwrap_or_default()
            },
            |prepare| {
                prepare
                    .visible_entities
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>()
            },
        )
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

fn selection_entities(
    selections: &[crate::graphics::types::VirtualGeometryClusterSelection],
) -> HashSet<u64> {
    selections
        .iter()
        .map(|selection| selection.entity)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::build_mesh_draw_build_context;
    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
        RenderVirtualGeometryExtract, RenderVirtualGeometryInstance,
    };
    use crate::core::math::{Transform, UVec2, Vec3};
    use crate::graphics::types::{
        ViewportRenderFrame, VirtualGeometryClusterSelection, VirtualGeometryPrepareCluster,
        VirtualGeometryPrepareClusterState, VirtualGeometryPrepareDrawSegment,
        VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
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

    #[test]
    fn build_context_prefers_explicit_cluster_selection_entities_over_prepare_visibility() {
        let frame = ViewportRenderFrame::from_extract(
            World::new().to_render_frame_extract(),
            UVec2::new(64, 64),
        )
        .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
            visible_entities: vec![111_u64],
            visible_clusters: Vec::new(),
            cluster_draw_segments: Vec::new(),
            resident_pages: Vec::new(),
            pending_page_requests: Vec::new(),
            available_slots: Vec::new(),
            evictable_pages: Vec::new(),
        }))
        .with_virtual_geometry_cluster_selections(Some(vec![VirtualGeometryClusterSelection {
            submission_index: 0,
            instance_index: Some(0),
            entity: 222_u64,
            cluster_id: 10,
            cluster_ordinal: 0,
            page_id: 20,
            lod_level: 1,
            submission_page_id: 20,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 0,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 1,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: Some(1),
            submission_slot: Some(1),
            state: VirtualGeometryPrepareClusterState::Resident,
        }]));

        let build_context = build_mesh_draw_build_context(&frame, true);

        assert_eq!(
            build_context.allowed_virtual_geometry_entities,
            Some(HashSet::from([222_u64])),
            "expected explicit frame-owned cluster selections to remain the authoritative entity filter even when prepare visibility is also present"
        );
    }

    #[test]
    fn build_context_keeps_prepare_visibility_when_frame_owned_selections_only_mirror_prepare_truth(
    ) {
        let mut extract = World::new().to_render_frame_extract();
        extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
            cluster_budget: 1,
            page_budget: 1,
            clusters: vec![RenderVirtualGeometryCluster {
                entity: 222_u64,
                cluster_id: 10,
                page_id: 20,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            }],
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity: 222_u64,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 1,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("BuildContextPrepareMirrorUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState::default(),
        });
        let prepare = VirtualGeometryPrepareFrame {
            visible_entities: vec![111_u64],
            visible_clusters: vec![VirtualGeometryPrepareCluster {
                entity: 222_u64,
                cluster_id: 10,
                page_id: 20,
                lod_level: 1,
                resident_slot: Some(1),
                state: VirtualGeometryPrepareClusterState::Resident,
            }],
            cluster_draw_segments: vec![VirtualGeometryPrepareDrawSegment {
                entity: 222_u64,
                cluster_id: 10,
                page_id: 20,
                resident_slot: Some(1),
                cluster_ordinal: 0,
                cluster_span_count: 1,
                cluster_count: 1,
                lineage_depth: 0,
                lod_level: 1,
                state: VirtualGeometryPrepareClusterState::Resident,
            }],
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 20,
                slot: 1,
                size_bytes: 4096,
            }],
            pending_page_requests: Vec::new(),
            available_slots: Vec::new(),
            evictable_pages: Vec::new(),
        };
        let prepare_owned_selections = prepare.cluster_selections(
            extract
                .geometry
                .virtual_geometry
                .as_ref()
                .expect("expected VG extract"),
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
            .with_virtual_geometry_prepare(Some(prepare))
            .with_prepare_derived_virtual_geometry_cluster_selections(Some(
                prepare_owned_selections,
            ));

        let build_context = build_mesh_draw_build_context(&frame, true);

        assert_eq!(
            build_context.allowed_virtual_geometry_entities,
            Some(HashSet::from([111_u64])),
            "expected runtime-frame selections that merely mirror prepare-owned truth to keep prepare visibility as the authoritative mesh-build entity gate"
        );
    }
}
