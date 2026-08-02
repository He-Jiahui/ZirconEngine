use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{
    ProjectionMode, RenderFrameExtract, RenderHybridGiExtract, RenderVirtualGeometryExtract,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{is_finite_vec3, Real};
use crate::core::TaskPool;

use super::super::super::culling::parallel_frustum::{
    mesh_frustum_visibility, MeshFrustumCandidate,
};
use super::super::super::declarations::{
    VisibilityBounds, VisibilityBvhInstance, VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy,
    VisibilityContext, VisibilityHistorySnapshot, VisibilityHybridGiProbe,
    VisibilityRelevanceEntry,
};
use super::super::super::planning::{
    build_bvh_update_plan::build_bvh_update_plan, build_draw_commands::build_draw_commands,
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
const STATIC_INDEX_PREFILTER_MAX_CELL_COUNT: usize = 4_096;

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
        Self::from_extract_with_history_static_index_and_task_pool(
            value,
            previous,
            previous_static_index,
            None,
        )
    }

    pub(crate) fn from_extract_with_history_static_index_and_task_pool(
        value: &RenderFrameExtract,
        previous: Option<&VisibilityHistorySnapshot>,
        previous_static_index: Option<&VisibilityStaticIndex>,
        task_pool: Option<&TaskPool>,
    ) -> Self {
        Self::from_extract_with_history_static_index_task_pool_and_feature_payloads(
            value,
            previous,
            previous_static_index,
            None,
            task_pool,
            value.lighting.hybrid_global_illumination.as_ref(),
            value.geometry.virtual_geometry.as_ref(),
        )
    }

    pub(crate) fn from_extract_with_history_static_index_task_pool_and_feature_payloads(
        value: &RenderFrameExtract,
        previous: Option<&VisibilityHistorySnapshot>,
        previous_static_index: Option<&VisibilityStaticIndex>,
        previous_dynamic_index: Option<&VisibilityStaticIndex>,
        task_pool: Option<&TaskPool>,
        _hybrid_global_illumination: Option<&RenderHybridGiExtract>,
        virtual_geometry: Option<&RenderVirtualGeometryExtract>,
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

        let bvh_update_plan = build_bvh_update_plan(&history_entries, previous);
        let static_index_instances = static_bvh_instances(&bvh_instances);
        let dynamic_index_instances = dynamic_bvh_instances(&bvh_instances);
        let (static_index, mut static_index_report) = build_static_index(
            previous_static_index,
            &static_index_instances,
            &bvh_update_plan,
        );
        let dynamic_index = build_dynamic_index(
            previous_dynamic_index,
            &dynamic_index_instances,
            &bvh_update_plan,
        );
        let main_view_culling = cull_main_view_with_static_index(
            value,
            &bvh_instances,
            &primitive_relevance,
            &static_index,
            task_pool,
        );
        static_index_report.main_view_prefilter_used = main_view_culling.prefilter_used;
        static_index_report.main_view_static_input_count = main_view_culling.static_input_count;
        static_index_report.main_view_static_candidate_count =
            main_view_culling.static_candidate_count;
        let frame_visibility = FrameVisibility::from_frame_views(
            &value.view.camera,
            value.view.scene_camera_entity,
            &value.view.cameras,
            &value.lighting,
            &bvh_instances,
            &primitive_relevance,
            &main_view_culling.visible_stable_instance_keys,
            task_pool,
        );
        let main_view_visible_stable_instance_keys =
            frame_visibility.main_view_visible_stable_instance_key_set();
        let visible_batches = Self::visible_batches_for_stable_instance_keys(
            &batches,
            &main_view_visible_stable_instance_keys,
        );
        let (visible_instances, draw_commands) = build_draw_commands(&visible_batches);
        let hybrid_gi_active_probes: Vec<VisibilityHybridGiProbe> = Vec::new();
        let hybrid_gi_update_plan = Default::default();
        let hybrid_gi_feedback = Default::default();
        let hybrid_gi_requested_probes: Vec<u32> = Vec::new();
        let (
            virtual_geometry_visible_clusters,
            virtual_geometry_draw_segments,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            virtual_geometry_requested_pages,
            virtual_geometry_history_visible_cluster_ids,
        ) = build_virtual_geometry_plan(
            virtual_geometry,
            &frame_visibility.main_view_visible_stable_instance_key_set(),
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
            primitive_relevance,
            batches,
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
            dynamic_index,
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

fn dynamic_bvh_instances(instances: &[VisibilityBvhInstance]) -> Vec<VisibilityBvhInstance> {
    instances
        .iter()
        .filter(|instance| matches!(instance.key.mobility, Mobility::Dynamic))
        .cloned()
        .collect()
}

fn build_static_index(
    previous_static_index: Option<&VisibilityStaticIndex>,
    static_instances: &[VisibilityBvhInstance],
    bvh_update_plan: &VisibilityBvhUpdatePlan,
) -> (VisibilityStaticIndex, VisibilityStaticIndexReport) {
    let (index, report) =
        build_spatial_index(previous_static_index, static_instances, bvh_update_plan);
    (index, report)
}

fn build_dynamic_index(
    previous_dynamic_index: Option<&VisibilityStaticIndex>,
    dynamic_instances: &[VisibilityBvhInstance],
    bvh_update_plan: &VisibilityBvhUpdatePlan,
) -> VisibilityStaticIndex {
    build_spatial_index(previous_dynamic_index, dynamic_instances, bvh_update_plan).0
}

fn build_spatial_index(
    previous_index: Option<&VisibilityStaticIndex>,
    instances: &[VisibilityBvhInstance],
    bvh_update_plan: &VisibilityBvhUpdatePlan,
) -> (VisibilityStaticIndex, VisibilityStaticIndexReport) {
    let mut index = previous_index.cloned().unwrap_or_default();
    if previous_index.is_none() {
        if instances.is_empty() {
            let report = index.report();
            return (index, report);
        }
        let report = index.rebuild(instances);
        return (index, report);
    }

    let report = if matches!(
        bvh_update_plan.strategy,
        VisibilityBvhUpdateStrategy::FullRebuild
    ) {
        index.rebuild(instances)
    } else {
        index.apply_update_plan(instances, bvh_update_plan)
    };
    (index, report)
}

struct MainViewCullingResult {
    visible_stable_instance_keys: BTreeSet<u64>,
    prefilter_used: bool,
    static_input_count: usize,
    static_candidate_count: usize,
}

fn cull_main_view_with_static_index(
    value: &RenderFrameExtract,
    bvh_instances: &[VisibilityBvhInstance],
    primitive_relevance: &[VisibilityRelevanceEntry],
    static_index: &VisibilityStaticIndex,
    task_pool: Option<&TaskPool>,
) -> MainViewCullingResult {
    let relevance_by_stable_instance_key = primitive_relevance
        .iter()
        .map(|entry| (entry.stable_instance_key, entry.relevance))
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
            let relevance = relevance_by_stable_instance_key
                .get(&instance.stable_instance_key)
                .copied()
                .unwrap_or_default();
            if !relevance.main_view() {
                return None;
            }
            if matches!(instance.key.mobility, Mobility::Static)
                && static_prefilter_candidates
                    .as_ref()
                    .is_some_and(|keys| !keys.contains(&instance.stable_instance_key))
            {
                return None;
            }
            Some(MeshFrustumCandidate {
                stable_instance_key: instance.stable_instance_key,
                bounds: instance.bounds,
            })
        })
        .collect::<Vec<_>>();
    let visible_stable_instance_keys =
        mesh_frustum_visibility(&candidates, &value.view.camera, task_pool)
            .into_iter()
            .filter_map(|entry| entry.visible.then_some(entry.stable_instance_key))
            .collect::<BTreeSet<_>>();

    MainViewCullingResult {
        visible_stable_instance_keys,
        prefilter_used: static_prefilter_candidates.is_some(),
        static_input_count,
        static_candidate_count,
    }
}

fn static_index_prefilter_candidates(
    static_index: &VisibilityStaticIndex,
    camera: &ViewportCameraSnapshot,
    static_input_count: usize,
) -> Option<BTreeSet<u64>> {
    if static_input_count < STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES {
        return None;
    }
    let query_bounds = conservative_camera_query_bounds(camera)?;
    static_index
        .query_bounds_with_stats_limited(query_bounds, STATIC_INDEX_PREFILTER_MAX_CELL_COUNT)
        .map(|query| query.stable_instance_keys.into_iter().collect())
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

#[cfg(test)]
mod tests;
