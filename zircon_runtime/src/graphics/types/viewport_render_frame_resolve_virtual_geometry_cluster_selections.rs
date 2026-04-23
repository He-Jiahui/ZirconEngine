use crate::core::framework::render::RenderVirtualGeometryClusterSelectionInputSource;

use std::borrow::Cow;

use super::viewport_render_frame::{
    ViewportRenderFrame, ViewportVirtualGeometryClusterSelectionSource,
};
use super::virtual_geometry_cluster_selection::VirtualGeometryClusterSelection;

impl ViewportRenderFrame {
    pub(crate) fn has_explicit_virtual_geometry_cluster_selection_override(&self) -> bool {
        matches!(
            self.virtual_geometry_cluster_selections_source,
            Some(ViewportVirtualGeometryClusterSelectionSource::ExplicitFrameOwned)
        )
    }

    pub(crate) fn virtual_geometry_cluster_selection_input_source(
        &self,
    ) -> RenderVirtualGeometryClusterSelectionInputSource {
        match self.virtual_geometry_cluster_selections_source {
            Some(ViewportVirtualGeometryClusterSelectionSource::ExplicitFrameOwned) => {
                RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned
            }
            Some(ViewportVirtualGeometryClusterSelectionSource::PrepareDerivedFrameOwned) => {
                RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned
            }
            None if self.virtual_geometry_prepare.is_some()
                && self.extract.geometry.virtual_geometry.is_some() =>
            {
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand
            }
            None => RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        }
    }

    pub(crate) fn resolved_virtual_geometry_cluster_selections(
        &self,
    ) -> Option<Cow<'_, [VirtualGeometryClusterSelection]>> {
        self.virtual_geometry_cluster_selections
            .as_deref()
            .map(Cow::Borrowed)
            .or_else(|| {
                self.virtual_geometry_prepare.as_ref().and_then(|prepare| {
                    self.extract
                        .geometry
                        .virtual_geometry
                        .as_ref()
                        .map(|extract| Cow::Owned(prepare.cluster_selections(extract)))
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryClusterSelectionInputSource,
        RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
        RenderVirtualGeometryInstance,
    };
    use crate::core::math::{Transform, UVec2, Vec3};
    use crate::graphics::types::{
        ViewportRenderFrame, VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    };
    use crate::scene::world::World;

    #[test]
    fn resolved_virtual_geometry_cluster_selections_prefers_explicit_frame_owned_input() {
        let frame =
            prepare_backed_viewport_frame().with_virtual_geometry_cluster_selections(Some(vec![
                VirtualGeometryClusterSelection {
                    submission_index: 7,
                    instance_index: Some(3),
                    entity: 9001,
                    cluster_id: 42,
                    cluster_ordinal: 5,
                    page_id: 400,
                    lod_level: 9,
                    submission_page_id: 400,
                    submission_lod_level: 9,
                    entity_cluster_start_ordinal: 5,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 6,
                    lineage_depth: 2,
                    frontier_rank: 8,
                    resident_slot: Some(4),
                    submission_slot: Some(4),
                    state: VirtualGeometryPrepareClusterState::Resident,
                },
            ]));

        let resolved = frame
            .resolved_virtual_geometry_cluster_selections()
            .expect("expected explicit frame-owned selections to resolve");

        assert!(
            matches!(resolved, Cow::Borrowed(_)),
            "expected explicit frame-owned selections to stay borrowed instead of rebuilding from prepare"
        );
        assert_eq!(
            resolved.as_ref(),
            &[VirtualGeometryClusterSelection {
                submission_index: 7,
                instance_index: Some(3),
                entity: 9001,
                cluster_id: 42,
                cluster_ordinal: 5,
                page_id: 400,
                lod_level: 9,
                submission_page_id: 400,
                submission_lod_level: 9,
                entity_cluster_start_ordinal: 5,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 6,
                lineage_depth: 2,
                frontier_rank: 8,
                resident_slot: Some(4),
                submission_slot: Some(4),
                state: VirtualGeometryPrepareClusterState::Resident,
            }],
            "expected explicit frame-owned VG selections to remain authoritative over prepare-derived fallback"
        );
    }

    #[test]
    fn resolved_virtual_geometry_cluster_selections_derives_from_prepare_when_explicit_input_absent(
    ) {
        let frame = prepare_backed_viewport_frame();
        let expected = frame
            .virtual_geometry_prepare
            .as_ref()
            .expect("expected prepare-backed frame")
            .cluster_selections(
                frame
                    .extract
                    .geometry
                    .virtual_geometry
                    .as_ref()
                    .expect("expected VG extract"),
            );

        let resolved = frame
            .resolved_virtual_geometry_cluster_selections()
            .expect("expected prepare-backed frame to synthesize selections");

        assert!(
            matches!(resolved, Cow::Owned(_)),
            "expected prepare-backed fallback to materialize owned selections"
        );
        assert_eq!(
            resolved.as_ref(),
            expected.as_slice(),
            "expected resolved selections to match the prepare+extract derived cluster worklist"
        );
    }

    #[test]
    fn resolved_virtual_geometry_cluster_selections_returns_none_without_inputs() {
        let world = World::new();
        let frame =
            ViewportRenderFrame::from_extract(world.to_render_frame_extract(), UVec2::new(64, 64));

        assert!(
            frame.resolved_virtual_geometry_cluster_selections().is_none(),
            "expected no resolved selections when the frame carries neither explicit selections nor prepare-owned VG truth"
        );
    }

    #[test]
    fn prepare_derived_frame_owned_cluster_selections_are_not_treated_as_explicit_override() {
        let frame = prepare_backed_viewport_frame()
            .with_prepare_derived_virtual_geometry_cluster_selections(Some(vec![
                VirtualGeometryClusterSelection {
                    submission_index: 0,
                    instance_index: Some(0),
                    entity: 9001,
                    cluster_id: 42,
                    cluster_ordinal: 0,
                    page_id: 400,
                    lod_level: 9,
                    submission_page_id: 400,
                    submission_lod_level: 9,
                    entity_cluster_start_ordinal: 0,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 1,
                    lineage_depth: 0,
                    frontier_rank: 0,
                    resident_slot: Some(4),
                    submission_slot: Some(4),
                    state: VirtualGeometryPrepareClusterState::Resident,
                },
            ]));

        assert!(
            !frame.has_explicit_virtual_geometry_cluster_selection_override(),
            "expected prepare-derived frame-owned cluster selections to preserve prepare authority instead of masquerading as an explicit override"
        );
        assert_eq!(
            frame.virtual_geometry_cluster_selection_input_source(),
            RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned,
            "expected prepare-derived frame-owned cluster selections to keep their provenance so later debug surfaces can distinguish mirrored prepare truth from explicit frame overrides"
        );
    }

    #[test]
    fn cluster_selection_input_source_reports_explicit_prepare_on_demand_and_unavailable() {
        let explicit =
            prepare_backed_viewport_frame().with_virtual_geometry_cluster_selections(Some(vec![
                VirtualGeometryClusterSelection {
                    submission_index: 0,
                    instance_index: Some(0),
                    entity: 9001,
                    cluster_id: 42,
                    cluster_ordinal: 0,
                    page_id: 400,
                    lod_level: 9,
                    submission_page_id: 400,
                    submission_lod_level: 9,
                    entity_cluster_start_ordinal: 0,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 1,
                    lineage_depth: 0,
                    frontier_rank: 0,
                    resident_slot: Some(4),
                    submission_slot: Some(4),
                    state: VirtualGeometryPrepareClusterState::Resident,
                },
            ]));
        let prepare_on_demand = prepare_backed_viewport_frame();
        let unavailable = ViewportRenderFrame::from_extract(
            World::new().to_render_frame_extract(),
            UVec2::new(64, 64),
        );

        assert_eq!(
            explicit.virtual_geometry_cluster_selection_input_source(),
            RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned,
            "expected explicit frame-owned selections to preserve explicit override provenance"
        );
        assert_eq!(
            prepare_on_demand.virtual_geometry_cluster_selection_input_source(),
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
            "expected prepare-backed frames without stored selections to report on-demand prepare provenance"
        );
        assert_eq!(
            unavailable.virtual_geometry_cluster_selection_input_source(),
            RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
            "expected frames without explicit selections or prepare-backed VG truth to report unavailable provenance"
        );
    }

    fn prepare_backed_viewport_frame() -> ViewportRenderFrame {
        let world = World::new();
        let entity = world
            .nodes()
            .iter()
            .find(|node| node.mesh.is_some())
            .map(|node| node.id)
            .expect("default world should contain a renderable mesh");
        let mut extract = world.to_render_frame_extract();
        extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
            cluster_budget: 2,
            page_budget: 1,
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 1,
                    page_id: 10,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 2,
                    page_id: 20,
                    lod_level: 10,
                    parent_cluster_id: Some(1),
                    bounds_center: Vec3::X,
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
            ],
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("ViewportFrameResolveSelectionsUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState::default(),
        });
        ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
            .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                visible_entities: vec![entity],
                visible_clusters: Vec::new(),
                cluster_draw_segments: Vec::new(),
                resident_pages: vec![VirtualGeometryPreparePage {
                    page_id: 10,
                    slot: 0,
                    size_bytes: 4096,
                }],
                pending_page_requests: Vec::new(),
                available_slots: Vec::new(),
                evictable_pages: Vec::new(),
            }))
    }
}
