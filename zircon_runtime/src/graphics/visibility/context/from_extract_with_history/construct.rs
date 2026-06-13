use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{ProjectionMode, RenderFrameExtract, ViewportCameraSnapshot};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{is_finite_vec3, Real};

use super::super::super::culling::parallel_frustum::{
    mesh_frustum_visibility, MeshFrustumCandidate,
};
use super::super::super::declarations::{
    VisibilityBatch, VisibilityBounds, VisibilityBvhInstance, VisibilityBvhUpdatePlan,
    VisibilityBvhUpdateStrategy, VisibilityContext, VisibilityHistorySnapshot,
    VisibilityRelevanceEntry,
};
use super::super::super::planning::{
    build_bvh_update_plan::build_bvh_update_plan, build_draw_commands::build_draw_commands,
    build_hybrid_gi_plan::build_hybrid_gi_plan,
    build_instance_upload_plan::build_instance_upload_plan,
    build_particle_upload_plan::build_particle_upload_plan,
    build_virtual_geometry_plan::build_virtual_geometry_plan,
};
use super::super::super::view_context::FrameVisibility;
use super::super::super::{VisibilityStaticIndex, VisibilityStaticIndexReport};
use super::batching_result::BatchingResult;
use super::build_history_snapshot::build_history_snapshot;
use super::collect_batching_result::collect_batching_result;
use super::collect_gpu_instancing_candidates::collect_gpu_instancing_candidates;

const STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES: usize = 10_000;

impl VisibilityContext {
    pub fn from_extract_with_history(
        value: &RenderFrameExtract,
        previous: Option<&VisibilityHistorySnapshot>,
    ) -> Self {
        Self::from_extract_with_history_and_static_index(value, previous, None)
    }

    pub(crate) fn from_extract_with_history_and_static_index(
        value: &RenderFrameExtract,
        previous: Option<&VisibilityHistorySnapshot>,
        previous_static_index: Option<&VisibilityStaticIndex>,
    ) -> Self {
        let BatchingResult {
            renderable_entities,
            static_entities,
            dynamic_entities,
            primitive_relevance,
            batches,
            bvh_instances,
            history_entries,
        } = collect_batching_result(value);

        let bvh_history_snapshot = VisibilityHistorySnapshot {
            instances: history_entries.clone(),
            ..VisibilityHistorySnapshot::default()
        };
        let bvh_update_plan = build_bvh_update_plan(&bvh_history_snapshot, previous);
        let static_index_instances = static_bvh_instances(&bvh_instances);
        let (static_index, mut static_index_report) = build_static_index(
            previous_static_index,
            &static_index_instances,
            &bvh_update_plan,
        );
        let main_view_culling = cull_main_view_with_static_index(
            value,
            &bvh_instances,
            &primitive_relevance,
            &static_index,
        );
        static_index_report.main_view_prefilter_used = main_view_culling.prefilter_used;
        static_index_report.main_view_static_input_count = main_view_culling.static_input_count;
        static_index_report.main_view_static_candidate_count =
            main_view_culling.static_candidate_count;
        let culled_entities = renderable_entities
            .difference(&main_view_culling.visible_entities)
            .copied()
            .collect::<BTreeSet<_>>();
        let frame_visibility = FrameVisibility::from_frame_views(
            &value.view.camera,
            &value.lighting,
            &bvh_instances,
            &primitive_relevance,
            &main_view_culling.visible_entities,
        );
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
            static_index_report,
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
            static_index,
        }
    }
}

fn static_bvh_instances(instances: &[VisibilityBvhInstance]) -> Vec<VisibilityBvhInstance> {
    instances
        .iter()
        .filter(|instance| matches!(instance.key.mobility, Mobility::Static))
        .cloned()
        .collect()
}

fn build_static_index(
    previous_static_index: Option<&VisibilityStaticIndex>,
    static_instances: &[VisibilityBvhInstance],
    bvh_update_plan: &VisibilityBvhUpdatePlan,
) -> (VisibilityStaticIndex, VisibilityStaticIndexReport) {
    let mut static_index = previous_static_index.cloned().unwrap_or_default();
    if previous_static_index.is_none() {
        if static_instances.is_empty() {
            let report = static_index.report();
            return (static_index, report);
        }
        let report = static_index.rebuild(static_instances);
        return (static_index, report);
    }

    let report = if matches!(
        bvh_update_plan.strategy,
        VisibilityBvhUpdateStrategy::FullRebuild
    ) {
        static_index.rebuild(static_instances)
    } else {
        static_index.apply_update_plan(static_instances, bvh_update_plan)
    };
    (static_index, report)
}

struct MainViewCullingResult {
    visible_entities: BTreeSet<EntityId>,
    prefilter_used: bool,
    static_input_count: usize,
    static_candidate_count: usize,
}

fn cull_main_view_with_static_index(
    value: &RenderFrameExtract,
    bvh_instances: &[VisibilityBvhInstance],
    primitive_relevance: &[VisibilityRelevanceEntry],
    static_index: &VisibilityStaticIndex,
) -> MainViewCullingResult {
    let relevance_by_entity = primitive_relevance
        .iter()
        .map(|entry| (entry.entity, entry.relevance))
        .collect::<BTreeMap<_, _>>();
    let static_input_count = bvh_instances
        .iter()
        .filter(|instance| matches!(instance.key.mobility, Mobility::Static))
        .count();
    let static_prefilter_candidates =
        static_index_prefilter_candidates(static_index, &value.view.camera, static_input_count);
    let static_candidate_count = static_prefilter_candidates
        .as_ref()
        .map_or(static_input_count, BTreeSet::len);
    let candidates = bvh_instances
        .iter()
        .filter_map(|instance| {
            let relevance = relevance_by_entity
                .get(&instance.entity)
                .copied()
                .unwrap_or_default();
            if !relevance.main_view() {
                return None;
            }
            if matches!(instance.key.mobility, Mobility::Static)
                && static_prefilter_candidates
                    .as_ref()
                    .is_some_and(|entities| !entities.contains(&instance.entity))
            {
                return None;
            }
            Some(MeshFrustumCandidate {
                entity: instance.entity,
                bounds: instance.bounds,
            })
        })
        .collect::<Vec<_>>();
    let visible_entities = mesh_frustum_visibility(&candidates, &value.view.camera)
        .into_iter()
        .filter_map(|entry| entry.visible.then_some(entry.entity))
        .collect::<BTreeSet<_>>();

    MainViewCullingResult {
        visible_entities,
        prefilter_used: static_prefilter_candidates.is_some(),
        static_input_count,
        static_candidate_count,
    }
}

fn static_index_prefilter_candidates(
    static_index: &VisibilityStaticIndex,
    camera: &ViewportCameraSnapshot,
    static_input_count: usize,
) -> Option<BTreeSet<EntityId>> {
    if static_input_count < STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES {
        return None;
    }
    let query_bounds = conservative_camera_query_bounds(camera)?;
    Some(
        static_index
            .query_bounds(query_bounds)
            .into_iter()
            .collect(),
    )
}

fn conservative_camera_query_bounds(camera: &ViewportCameraSnapshot) -> Option<VisibilityBounds> {
    let far = camera.z_far;
    if !far.is_finite() || far <= 0.0 || !is_finite_vec3(camera.transform.translation) {
        return None;
    }

    let radius = match camera.projection_mode {
        ProjectionMode::Perspective => {
            let half_fov_tan = (camera.fov_y_radians * 0.5).tan().abs();
            let aspect = camera.aspect_ratio.abs().max(1.0);
            far * (1.0 + half_fov_tan.powi(2) * (1.0 + aspect.powi(2))).sqrt()
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.abs();
            let half_width = half_height * camera.aspect_ratio.abs().max(1.0);
            (far.powi(2) + half_width.powi(2) + half_height.powi(2)).sqrt()
        }
    };

    radius.is_finite().then_some(VisibilityBounds {
        center: camera.transform.translation,
        radius: radius.max(0.0) as Real,
    })
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
        LightShadowSettings, LightingExtract, ParticleExtract, PostProcessExtract,
        RenderDirectionalLightSnapshot, RenderFrameExtract, RenderMaterialAlphaMode,
        RenderMeshSnapshot, RenderMeshStaticState, RenderOverlayExtract, RenderPointLightSnapshot,
        RenderSpotLightSnapshot, RenderViewExtract, RenderWorldSnapshotHandle, ShadowPcfQuality,
        ShadowResolutionTier, SpriteExtract, ViewportCameraSnapshot,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Real, Transform, Vec3, Vec4};
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
                    light_id: 10,
                    layer_mask: 1,
                    direction: Vec3::new(0.0, -1.0, -1.0),
                    color: Vec3::ONE,
                    intensity: 1.0,
                    shadow: None,
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

    #[test]
    fn visibility_context_builds_shadow_views_for_atlas_light_slots() {
        let camera = ViewportCameraSnapshot::default();
        let frame = RenderFrameExtract {
            world: RenderWorldSnapshotHandle::new(1),
            view: RenderViewExtract::from_camera(camera),
            geometry: GeometryExtract::from_meshes_and_phase_inputs(
                CorePipelineKind::Core3d,
                vec![mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1)],
                vec![GeometryPhaseInput::new(
                    1,
                    0,
                    RenderMaterialAlphaMode::Opaque,
                    -5.0,
                )],
            ),
            animation_poses: Vec::new(),
            lighting: LightingExtract {
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 10,
                    light_id: 10,
                    layer_mask: 1,
                    direction: Vec3::new(0.0, -1.0, -1.0),
                    color: Vec3::ONE,
                    intensity: 1.0,
                    shadow: Some(shadow_settings()),
                }],
                point_lights: vec![RenderPointLightSnapshot {
                    node_id: 20,
                    light_id: 20,
                    layer_mask: 1,
                    position: Vec3::ZERO,
                    color: Vec3::ONE,
                    intensity: 1.0,
                    range: 8.0,
                    shadow: Some(shadow_settings()),
                }],
                spot_lights: vec![RenderSpotLightSnapshot {
                    node_id: 30,
                    light_id: 30,
                    layer_mask: 1,
                    position: Vec3::ZERO,
                    direction: Vec3::new(0.0, 0.0, -1.0),
                    color: Vec3::ONE,
                    intensity: 1.0,
                    range: 8.0,
                    inner_angle_radians: 0.25,
                    outer_angle_radians: 0.5,
                    shadow: Some(shadow_settings()),
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

        assert_eq!(context.frame_visibility.shadow_views().count(), 11);
        for cascade in 0..4 {
            assert!(context
                .frame_visibility
                .view(&VisibilityViewKey::ShadowCascade { light: 10, cascade })
                .is_some());
        }
        let first_cascade = context
            .frame_visibility
            .view(&VisibilityViewKey::ShadowCascade {
                light: 10,
                cascade: 0,
            })
            .unwrap();
        let last_cascade = context
            .frame_visibility
            .view(&VisibilityViewKey::ShadowCascade {
                light: 10,
                cascade: 3,
            })
            .unwrap();
        assert!(last_cascade.camera.ortho_size > first_cascade.camera.ortho_size);
        assert_ne!(
            last_cascade.camera.transform.translation,
            first_cascade.camera.transform.translation
        );
        for face in 0..6 {
            assert!(context
                .frame_visibility
                .view(&VisibilityViewKey::ShadowPointFace { light: 20, face })
                .is_some());
        }
        assert!(context
            .frame_visibility
            .view(&VisibilityViewKey::ShadowSpot { light: 30 })
            .is_some());
    }

    #[test]
    fn visibility_context_reuses_static_index_without_frame_rebuild() {
        let frame = frame_from_meshes(vec![
            mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
            mesh_at(2, Vec3::new(32.0, 0.0, -5.0), 1),
        ]);
        let first =
            VisibilityContext::from_extract_with_history_and_static_index(&frame, None, None);
        let previous_history = first.history_snapshot.clone();
        let previous_static_index = first.static_index().clone();

        let second = VisibilityContext::from_extract_with_history_and_static_index(
            &frame,
            Some(&previous_history),
            Some(&previous_static_index),
        );

        assert_eq!(first.static_index_report.frame_full_rebuild_count, 1);
        assert_eq!(first.static_index_report.indexed_entity_count, 2);
        assert_eq!(second.static_index_report.frame_full_rebuild_count, 0);
        assert_eq!(second.static_index_report.frame_incremental_update_count, 1);
        assert_eq!(second.static_index_report.indexed_entity_count, 2);
    }

    #[test]
    fn visibility_context_rebuilds_static_index_when_previous_index_is_missing() {
        let frame = frame_from_meshes(vec![
            mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
            mesh_at(2, Vec3::new(32.0, 0.0, -5.0), 1),
        ]);
        let first =
            VisibilityContext::from_extract_with_history_and_static_index(&frame, None, None);

        let second = VisibilityContext::from_extract_with_history_and_static_index(
            &frame,
            Some(&first.history_snapshot),
            None,
        );

        assert_eq!(second.static_index_report.frame_full_rebuild_count, 1);
        assert_eq!(second.static_index_report.frame_incremental_update_count, 0);
        assert_eq!(second.static_index_report.indexed_entity_count, 2);
    }

    #[test]
    fn visibility_context_uses_static_index_prefilter_above_threshold() {
        let mut meshes = Vec::with_capacity(super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES + 1);
        meshes.push(mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1));
        for index in 0..super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES {
            meshes.push(mesh_at(
                1_000 + index as u64,
                Vec3::new(10_000.0 + index as Real * 32.0, 0.0, -5.0),
                1,
            ));
        }
        let frame = frame_from_meshes(meshes);

        let context =
            VisibilityContext::from_extract_with_history_and_static_index(&frame, None, None);

        assert!(context.static_index_report.main_view_prefilter_used);
        assert_eq!(
            context.static_index_report.main_view_static_input_count,
            super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES + 1
        );
        assert!(
            context.static_index_report.main_view_static_candidate_count
                < context.static_index_report.main_view_static_input_count
        );
        assert_eq!(context.visible_entities, vec![1]);
    }

    fn frame_from_meshes(meshes: Vec<RenderMeshSnapshot>) -> RenderFrameExtract {
        let phase_inputs = meshes
            .iter()
            .enumerate()
            .map(|(index, mesh)| {
                GeometryPhaseInput::new(mesh.node_id, index, RenderMaterialAlphaMode::Opaque, -5.0)
            })
            .collect::<Vec<_>>();

        RenderFrameExtract {
            world: RenderWorldSnapshotHandle::new(1),
            view: RenderViewExtract::from_camera(ViewportCameraSnapshot::default()),
            geometry: GeometryExtract::from_meshes_and_phase_inputs(
                CorePipelineKind::Core3d,
                meshes,
                phase_inputs,
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
        }
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

    fn shadow_settings() -> LightShadowSettings {
        LightShadowSettings {
            casts_shadow: true,
            depth_bias: 0.0,
            normal_bias: 0.0,
            strength: 1.0,
            resolution_preference: ShadowResolutionTier::T512,
            pcf_quality: ShadowPcfQuality::High,
        }
    }
}
