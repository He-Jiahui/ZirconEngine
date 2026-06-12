use std::collections::BTreeSet;

use crate::core::framework::render::RenderFrameExtract;
use crate::core::framework::scene::EntityId;

use super::super::super::declarations::{
    VisibilityBatch, VisibilityContext, VisibilityHistorySnapshot,
};
use super::super::super::planning::{
    build_bvh_update_plan::build_bvh_update_plan, build_draw_commands::build_draw_commands,
    build_hybrid_gi_plan::build_hybrid_gi_plan,
    build_instance_upload_plan::build_instance_upload_plan,
    build_particle_upload_plan::build_particle_upload_plan,
    build_virtual_geometry_plan::build_virtual_geometry_plan,
};
use super::batching_result::BatchingResult;
use super::build_history_snapshot::build_history_snapshot;
use super::collect_batching_result::collect_batching_result;
use super::collect_gpu_instancing_candidates::collect_gpu_instancing_candidates;

impl VisibilityContext {
    pub fn from_extract_with_history(
        value: &RenderFrameExtract,
        previous: Option<&VisibilityHistorySnapshot>,
    ) -> Self {
        let BatchingResult {
            frame_visibility,
            renderable_entities,
            static_entities,
            dynamic_entities,
            visible_entities: _visible_entities,
            culled_entities,
            primitive_relevance,
            batches,
            bvh_instances,
            history_entries,
        } = collect_batching_result(value);

        let main_view_visible_entities = frame_visibility.main_view_visible_entity_set();
        let visible_batches = visible_batches_for_view(&batches, &main_view_visible_entities);
        let (visible_instances, draw_commands) = build_draw_commands(&visible_batches);
        let (
            hybrid_gi_active_probes,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            hybrid_gi_requested_probes,
        ) = build_hybrid_gi_plan(
            value.lighting.hybrid_global_illumination.as_ref(),
            &main_view_visible_entities,
            &value.view.camera,
            previous,
        );
        let (
            virtual_geometry_visible_clusters,
            virtual_geometry_draw_segments,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            virtual_geometry_requested_pages,
            virtual_geometry_history_visible_cluster_ids,
        ) = build_virtual_geometry_plan(
            value.geometry.virtual_geometry.as_ref(),
            &main_view_visible_entities,
            &value.view.camera,
            previous,
        );
        let history_snapshot = build_history_snapshot(
            value,
            history_entries,
            hybrid_gi_active_probes
                .iter()
                .map(|probe| probe.probe_id)
                .collect(),
            hybrid_gi_requested_probes,
            virtual_geometry_history_visible_cluster_ids,
            virtual_geometry_requested_pages,
        );
        let bvh_update_plan = build_bvh_update_plan(&history_snapshot, previous);
        let instance_upload_plan = build_instance_upload_plan(&bvh_instances, &bvh_update_plan);
        let particle_upload_plan = build_particle_upload_plan(&history_snapshot, previous);
        let gpu_instancing_candidates = collect_gpu_instancing_candidates(&visible_batches);

        Self {
            frame_visibility,
            renderable_entities: renderable_entities.into_iter().collect(),
            static_entities: static_entities.into_iter().collect(),
            dynamic_entities: dynamic_entities.into_iter().collect(),
            visible_entities: main_view_visible_entities.iter().copied().collect(),
            culled_entities: culled_entities.into_iter().collect(),
            primitive_relevance,
            batches,
            visible_batches,
            visible_instances,
            draw_commands,
            bvh_instances,
            bvh_update_plan,
            history_snapshot,
            instance_upload_plan,
            particle_upload_plan,
            hybrid_gi_active_probes,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            virtual_geometry_visible_clusters,
            virtual_geometry_draw_segments,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            gpu_instancing_candidates,
        }
    }
}

fn visible_batches_for_view(
    batches: &[VisibilityBatch],
    visible_entities: &BTreeSet<EntityId>,
) -> Vec<VisibilityBatch> {
    batches
        .iter()
        .filter_map(|batch| {
            let entities = batch
                .entities
                .iter()
                .copied()
                .filter(|entity| visible_entities.contains(entity))
                .collect::<Vec<_>>();
            (!entities.is_empty()).then_some(VisibilityBatch {
                key: batch.key,
                entities,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CorePipelineKind, DebugOverlayExtract, GeometryExtract, GeometryPhaseInput,
        LightingExtract, ParticleExtract, PostProcessExtract, RenderDirectionalLightSnapshot,
        RenderFrameExtract, RenderMaterialAlphaMode, RenderMeshSnapshot, RenderMeshStaticState,
        RenderOverlayExtract, RenderViewExtract, RenderWorldSnapshotHandle, SpriteExtract,
        ViewportCameraSnapshot,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Transform, Vec3, Vec4};
    use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
    use crate::graphics::visibility::{VisibilityContext, VisibilityViewKey};

    #[test]
    fn visibility_context_records_relevance_and_filters_main_view_layers() {
        let camera = ViewportCameraSnapshot::default();
        let frame = RenderFrameExtract {
            world: RenderWorldSnapshotHandle::new(1),
            view: RenderViewExtract::from_camera(camera),
            geometry: GeometryExtract::from_meshes_and_phase_inputs(
                CorePipelineKind::Core3d,
                vec![
                    mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
                    mesh_at(2, Vec3::new(0.0, 0.0, -5.0), 1 << 4),
                ],
                vec![
                    GeometryPhaseInput::new(1, 0, RenderMaterialAlphaMode::Opaque, -5.0),
                    GeometryPhaseInput::new(
                        2,
                        1,
                        RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
                        -5.0,
                    ),
                ],
            ),
            animation_poses: Vec::new(),
            lighting: LightingExtract::default(),
            post_process: PostProcessExtract::default(),
            debug: DebugOverlayExtract {
                overlays: RenderOverlayExtract::default(),
            },
            sprites: SpriteExtract::default(),
            particles: ParticleExtract::default(),
            visibility: Default::default(),
        };

        let context = VisibilityContext::from_extract(&frame);

        assert_eq!(context.visible_entities, vec![1]);
        assert_eq!(context.culled_entities, vec![2]);
        assert_eq!(
            context
                .visible_batches
                .iter()
                .flat_map(|batch| batch.entities.iter().copied())
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(context.primitive_relevance.len(), 2);
        assert_eq!(context.frame_visibility.entities, vec![1, 2]);
        assert_eq!(context.frame_visibility.bounds.len(), 2);
        assert_eq!(context.frame_visibility.relevance.len(), 2);
        assert_eq!(
            context.frame_visibility.main_view_visible_entities(),
            vec![1]
        );

        let main_view = context.frame_visibility.main_view().unwrap();
        assert_eq!(main_view.view, VisibilityViewKey::MainCamera);
        assert_eq!(main_view.visible, vec![0]);
        assert_eq!(main_view.stats.input_count, 2);
        assert_eq!(main_view.stats.layer_filtered_count, 1);
        assert_eq!(main_view.stats.frustum_culled_count, 0);
        assert_eq!(main_view.stats.occlusion_culled_count, 0);
        assert_eq!(main_view.stats.visible_count, 1);
        assert!(context
            .frame_visibility
            .view(&VisibilityViewKey::ShadowCascade {
                light: 10,
                cascade: 0
            })
            .is_none());

        let visible = context
            .primitive_relevance
            .iter()
            .find(|entry| entry.entity == 1)
            .unwrap()
            .relevance;
        assert!(visible.main_view());
        assert!(visible.depth_prepass());

        let hidden = context
            .primitive_relevance
            .iter()
            .find(|entry| entry.entity == 2)
            .unwrap()
            .relevance;
        assert!(!hidden.main_view());
        assert!(!hidden.depth_prepass());
        assert!(hidden.shadow_caster());
    }

    #[test]
    fn visibility_context_builds_shadow_view_independent_from_main_layers() {
        let camera = ViewportCameraSnapshot::default();
        let frame = RenderFrameExtract {
            world: RenderWorldSnapshotHandle::new(1),
            view: RenderViewExtract::from_camera(camera),
            geometry: GeometryExtract::from_meshes_and_phase_inputs(
                CorePipelineKind::Core3d,
                vec![
                    mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
                    mesh_at(2, Vec3::new(0.0, 0.0, -5.0), 1 << 4),
                ],
                vec![
                    GeometryPhaseInput::new(1, 0, RenderMaterialAlphaMode::Opaque, -5.0),
                    GeometryPhaseInput::new(
                        2,
                        1,
                        RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
                        -5.0,
                    ),
                ],
            ),
            animation_poses: Vec::new(),
            lighting: LightingExtract {
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 10,
                    direction: Vec3::new(0.0, -1.0, -1.0),
                    color: Vec3::ONE,
                    intensity: 1.0,
                }],
                ..LightingExtract::default()
            },
            post_process: PostProcessExtract::default(),
            debug: DebugOverlayExtract {
                overlays: RenderOverlayExtract::default(),
            },
            sprites: SpriteExtract::default(),
            particles: ParticleExtract::default(),
            visibility: Default::default(),
        };

        let context = VisibilityContext::from_extract(&frame);

        assert_eq!(context.visible_entities, vec![1]);
        let shadow_view = context
            .frame_visibility
            .view(&VisibilityViewKey::ShadowCascade {
                light: 10,
                cascade: 0,
            })
            .unwrap();
        assert_eq!(shadow_view.visible, vec![0, 1]);
        assert_eq!(shadow_view.stats.input_count, 2);
        assert_eq!(shadow_view.stats.layer_filtered_count, 0);
        assert_eq!(shadow_view.stats.frustum_culled_count, 0);
        assert_eq!(shadow_view.stats.occlusion_culled_count, 0);
        assert_eq!(shadow_view.stats.visible_count, 2);
        assert_eq!(
            context
                .frame_visibility
                .shadow_visible_entity_set()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    fn mesh_at(node_id: u64, translation: Vec3, render_layer_mask: u32) -> RenderMeshSnapshot {
        let mut transform = Transform::default();
        transform.translation = translation;

        RenderMeshSnapshot {
            node_id,
            stable_instance_key: node_id << 16,
            transform_revision: 0,
            transform,
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("tests/model")),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "tests/material",
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Static,
            static_state: RenderMeshStaticState::default(),
            render_layer_mask,
        }
    }
}
