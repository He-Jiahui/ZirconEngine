use crate::graphics::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareSurfaceCachePageContent,
    HybridGiPrepareVoxelClipmap, HybridGiScenePrepareFrame,
};

use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn build_scene_prepare_frame(&self) -> HybridGiScenePrepareFrame {
        let scene_representation = self.scene_representation();
        let card_bounds_by_owner_card_id = scene_representation.card_bounds_by_id();
        HybridGiScenePrepareFrame {
            card_capture_requests: self
                .scene_representation()
                .card_capture_request_descriptors()
                .iter()
                .map(|request| HybridGiPrepareCardCaptureRequest {
                    card_id: request.card_id(),
                    page_id: request.page_id(),
                    atlas_slot_id: request.atlas_slot_id(),
                    capture_slot_id: request.capture_slot_id(),
                    bounds_center: request.bounds_center(),
                    bounds_radius: request.bounds_radius(),
                })
                .collect(),
            surface_cache_page_contents: self
                .scene_representation()
                .surface_cache()
                .page_contents_snapshot()
                .into_iter()
                .filter_map(
                    |(
                        page_id,
                        owner_card_id,
                        atlas_slot_id,
                        capture_slot_id,
                        atlas_sample_rgba,
                        capture_sample_rgba,
                    )| {
                        let (bounds_center, bounds_radius) =
                            card_bounds_by_owner_card_id.get(&owner_card_id).copied()?;
                        Some(HybridGiPrepareSurfaceCachePageContent {
                            page_id,
                            owner_card_id,
                            atlas_slot_id,
                            capture_slot_id,
                            bounds_center,
                            bounds_radius,
                            atlas_sample_rgba,
                            capture_sample_rgba,
                        })
                    },
                )
                .collect(),
            voxel_clipmaps: self
                .scene_representation()
                .voxel_scene()
                .clipmap_descriptors_snapshot()
                .into_iter()
                .map(
                    |(clipmap_id, center, half_extent)| HybridGiPrepareVoxelClipmap {
                        clipmap_id,
                        center,
                        half_extent,
                    },
                )
                .collect(),
            voxel_cells: self
                .scene_representation()
                .voxel_scene()
                .voxel_cells_snapshot(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{RenderHybridGiExtract, RenderMeshSnapshot};
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Transform, Vec3, Vec4};
    use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

    #[test]
    fn build_scene_prepare_frame_preserves_persisted_surface_cache_owner_card_id_when_page_id_differs(
    ) {
        let mut state = HybridGiRuntimeState::default();
        let extract = hybrid_gi_settings(1);
        let mesh = mesh_at(
            7,
            "res://materials/persisted-owner.mat",
            Vec3::new(3.0, 0.5, -1.0),
            2.0,
        );

        state.register_scene_extract(Some(&extract), &[mesh.clone()], &[], &[], &[]);
        state.register_scene_extract(Some(&extract), &[mesh], &[], &[], &[]);
        state
            .scene_representation_mut()
            .surface_cache_mut()
            .replace_page_contents_for_test(&[(22, 7, 0, 0, [10, 20, 30, 255], [40, 50, 60, 255])]);

        let frame = state.build_scene_prepare_frame();

        assert!(frame.card_capture_requests.is_empty());
        assert_eq!(
            frame.surface_cache_page_contents,
            vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 22,
                owner_card_id: 7,
                atlas_slot_id: 0,
                capture_slot_id: 0,
                bounds_center: Vec3::new(3.0, 0.5, -1.0),
                bounds_radius: 1.0,
                atlas_sample_rgba: [10, 20, 30, 255],
                capture_sample_rgba: [40, 50, 60, 255],
            }]
        );
    }

    fn hybrid_gi_settings(card_budget: u32) -> RenderHybridGiExtract {
        RenderHybridGiExtract {
            enabled: true,
            quality: Default::default(),
            trace_budget: 0,
            card_budget,
            voxel_budget: 0,
            debug_view: Default::default(),
            probe_budget: 0,
            tracing_budget: 0,
            probes: Vec::new(),
            trace_regions: Vec::new(),
        }
    }

    fn mesh_at(
        entity: u64,
        material: &str,
        translation: Vec3,
        uniform_scale: f32,
    ) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id: entity,
            transform: Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/card.obj",
            )),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                material,
            )),
            tint: Vec4::ONE,
            mobility: Mobility::Static,
            render_layer_mask: u32::MAX,
        }
    }
}
