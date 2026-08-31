use super::super::super::graph_execution::RenderGraphExecutionRecord;
use super::super::super::mesh::PreparedMeshQueueStats;
use super::super::super::sprite::PreparedSpriteQueueStats;
use super::super::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;
use crate::core::framework::render::RenderCameraTargetGraphImportReport;
use crate::rhi::SubmissionTicket;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCompiledSceneOutputs {
    advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
    render_graph_execution: RenderGraphExecutionRecord,
    prepared_mesh_queue_stats: PreparedMeshQueueStats,
    prepared_sprite_queue_stats: PreparedSpriteQueueStats,
    scene_submission: SubmissionTicket,
    output_target_graph_import_report: Option<RenderCameraTargetGraphImportReport>,
}

impl SceneRendererCompiledSceneOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
        render_graph_execution: RenderGraphExecutionRecord,
        prepared_mesh_queue_stats: PreparedMeshQueueStats,
        prepared_sprite_queue_stats: PreparedSpriteQueueStats,
        scene_submission: SubmissionTicket,
    ) -> Self {
        Self {
            advanced_plugin_readbacks,
            render_graph_execution,
            prepared_mesh_queue_stats,
            prepared_sprite_queue_stats,
            scene_submission,
            output_target_graph_import_report: None,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn scene_submission(
        &self,
    ) -> SubmissionTicket {
        self.scene_submission
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn with_output_target_graph_import_report(
        mut self,
        report: RenderCameraTargetGraphImportReport,
    ) -> Self {
        self.output_target_graph_import_report = Some(report);
        self
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn output_target_graph_import_report(
        &self,
    ) -> Option<RenderCameraTargetGraphImportReport> {
        self.output_target_graph_import_report
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_parts(
        self,
    ) -> (
        SceneRendererAdvancedPluginReadbacks,
        RenderGraphExecutionRecord,
        PreparedMeshQueueStats,
        PreparedSpriteQueueStats,
        Option<RenderCameraTargetGraphImportReport>,
    ) {
        (
            self.advanced_plugin_readbacks,
            self.render_graph_execution,
            self.prepared_mesh_queue_stats,
            self.prepared_sprite_queue_stats,
            self.output_target_graph_import_report,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rhi::{DeviceGeneration, DeviceId, RenderQueueClass, SubmissionTicket};

    fn scene_submission() -> SubmissionTicket {
        SubmissionTicket::new(
            DeviceId::new(1),
            DeviceGeneration::initial(),
            RenderQueueClass::Graphics,
            2,
        )
    }

    #[test]
    fn compiled_scene_outputs_carry_prepared_mesh_queue_stats() {
        let stats = PreparedMeshQueueStats {
            draw_count: 3,
            early_z_draw_count: 2,
            shadow_caster_draw_count: 2,
            alpha_mask_shadow_caster_draw_count: 1,
            skinned_draw_count: 1,
            skinned_palette_upload_count: 1,
            gpu_instancing_candidate_group_count: 1,
            gpu_instancing_candidate_draw_count: 2,
            ..PreparedMeshQueueStats::default()
        };

        let outputs = SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::new(),
            RenderGraphExecutionRecord::default(),
            stats,
            PreparedSpriteQueueStats::default(),
            scene_submission(),
        );

        let (_readbacks, _record, carried_stats, _sprite_stats, _graph_import) =
            outputs.into_parts();
        assert_eq!(carried_stats, stats);
    }

    #[test]
    fn compiled_scene_outputs_carry_prepared_sprite_queue_stats() {
        let stats = PreparedSpriteQueueStats {
            draw_batch_count: 2,
            sprite_count: 3,
            image_slice_count: 5,
            expanded_image_slice_count: 2,
            vertex_count: 30,
            opaque_draw_batch_count: 1,
            transparent_draw_batch_count: 1,
            ..PreparedSpriteQueueStats::default()
        };

        let outputs = SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::new(),
            RenderGraphExecutionRecord::default(),
            PreparedMeshQueueStats::default(),
            stats,
            scene_submission(),
        );

        let (_readbacks, _record, _mesh_stats, carried_stats, _graph_import) = outputs.into_parts();
        assert_eq!(carried_stats, stats);
    }

    #[test]
    fn compiled_scene_outputs_can_carry_output_target_graph_import_report() {
        let report = RenderCameraTargetGraphImportReport::direct_imported(
            crate::core::math::UVec2::new(16, 9),
        );

        let outputs = SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::new(),
            RenderGraphExecutionRecord::default(),
            PreparedMeshQueueStats::default(),
            PreparedSpriteQueueStats::default(),
            scene_submission(),
        )
        .with_output_target_graph_import_report(report);

        let (_readbacks, _record, _mesh_stats, _sprite_stats, graph_import) = outputs.into_parts();
        assert_eq!(graph_import, Some(report));
    }
}
