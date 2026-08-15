use crate::hybrid_gi::{
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
            card_owner_stable_instance_keys: scene_representation.card_owner_stable_instance_keys(),
            radiance_cache_bootstrap_updates: scene_representation
                .radiance_cache_gpu_bootstrap_updates(),
            radiance_cache_updates: scene_representation.radiance_cache_gpu_updates(),
            radiance_cache_consumes: scene_representation.radiance_cache_gpu_consumes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::framework::render::{
        render_mesh_stable_instance_key, render_mesh_transform_revision, RenderHybridGiExtract,
        RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
    };
    use zircon_runtime::core::framework::scene::Mobility;
    use zircon_runtime::core::math::{Transform, Vec3, Vec4};
    use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

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

        state.register_scene_extract(Some(&extract), &[mesh.clone()], &[], &[], &[], None, false);
        state.register_scene_extract(Some(&extract), &[mesh], &[], &[], &[], None, false);
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

    #[test]
    fn build_scene_prepare_frame_exports_completed_radiance_cache_for_gpu_update_and_consume() {
        let mut state = HybridGiRuntimeState::default();
        let mut extract = hybrid_gi_settings(1);
        extract.trace_budget = 1;
        let mesh = mesh_at(
            7,
            "res://materials/radiance-cache-gpu-prepare.mat",
            Vec3::new(3.0, 0.5, -1.0),
            2.0,
        );

        state.register_scene_extract(Some(&extract), &[mesh.clone()], &[], &[], &[], None, false);
        let changed_frame = state.build_scene_prepare_frame();
        assert!(
            !changed_frame.radiance_cache_updates.is_empty(),
            "a completed changed generation must export bounded slot updates for GPU storage"
        );
        assert_eq!(
            changed_frame.radiance_cache_bootstrap_updates, changed_frame.radiance_cache_updates,
            "the first generation must be sufficient to bootstrap a fresh renderer instance"
        );
        assert!(
            !changed_frame.radiance_cache_consumes.is_empty(),
            "a completed changed generation must export the eight-corner screen-probe consume mapping"
        );
        assert!(changed_frame
            .radiance_cache_updates
            .iter()
            .all(|update| update.generation > 0));

        state.register_scene_extract(Some(&extract), &[mesh], &[], &[], &[], None, false);
        let stable_frame = state.build_scene_prepare_frame();
        assert!(
            stable_frame.radiance_cache_updates.is_empty(),
            "stable frames must retain the GPU RC storage instead of uploading an unchanged generation"
        );
        assert_eq!(
            stable_frame.radiance_cache_bootstrap_updates,
            changed_frame.radiance_cache_bootstrap_updates,
            "stable frames must retain a CPU-side recovery snapshot for renderer-instance eviction"
        );
        assert_eq!(
            stable_frame.radiance_cache_consumes, changed_frame.radiance_cache_consumes,
            "stable screen-probe consumption must retain the completed slot/weight mapping"
        );
    }

    fn hybrid_gi_settings(card_budget: u32) -> RenderHybridGiExtract {
        RenderHybridGiExtract {
            enabled: true,
            mode: Default::default(),
            profile: Default::default(),
            quality: Default::default(),
            trace_budget: 0,
            card_budget,
            voxel_budget: 0,
            debug_view: Default::default(),
        }
    }

    fn mesh_at(
        entity: u64,
        material: &str,
        translation: Vec3,
        uniform_scale: f32,
    ) -> RenderMeshSnapshot {
        let transform =
            Transform::from_translation(translation).with_scale(Vec3::splat(uniform_scale));
        RenderMeshSnapshot {
            node_id: entity,
            stable_instance_key: render_mesh_stable_instance_key(entity, 0),
            transform_revision: render_mesh_transform_revision(&transform),
            transform,
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/card.obj",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                material,
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Static,
            static_state: RenderMeshStaticState::from_transform_static(true),
            common: RendererCommon {
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                is_static: true,
                ..RendererCommon::default()
            },
        }
    }
}
