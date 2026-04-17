use zircon_scene::RenderFrameExtract;

use super::super::super::declarations::{VisibilityContext, VisibilityHistorySnapshot};
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
            renderable_entities,
            static_entities,
            dynamic_entities,
            visible_entities,
            culled_entities,
            batches,
            visible_batches,
            bvh_instances,
            history_entries,
        } = collect_batching_result(value);

        let (visible_instances, draw_commands) = build_draw_commands(&visible_batches);
        let (
            hybrid_gi_active_probes,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            hybrid_gi_requested_probes,
        ) = build_hybrid_gi_plan(
            value.lighting.hybrid_global_illumination.as_ref(),
            &visible_entities,
            &value.view.camera,
            previous,
        );
        let (
            virtual_geometry_visible_clusters,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            virtual_geometry_requested_pages,
        ) = build_virtual_geometry_plan(
            value.geometry.virtual_geometry.as_ref(),
            &visible_entities,
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
            virtual_geometry_visible_clusters
                .iter()
                .map(|cluster| cluster.cluster_id)
                .collect(),
            virtual_geometry_requested_pages,
        );
        let bvh_update_plan = build_bvh_update_plan(&history_snapshot, previous);
        let instance_upload_plan = build_instance_upload_plan(&bvh_instances, &bvh_update_plan);
        let particle_upload_plan = build_particle_upload_plan(&history_snapshot, previous);
        let gpu_instancing_candidates = collect_gpu_instancing_candidates(&visible_batches);

        Self {
            renderable_entities: renderable_entities.into_iter().collect(),
            static_entities: static_entities.into_iter().collect(),
            dynamic_entities: dynamic_entities.into_iter().collect(),
            visible_entities: visible_entities.into_iter().collect(),
            culled_entities: culled_entities.into_iter().collect(),
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
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            gpu_instancing_candidates,
        }
    }
}
