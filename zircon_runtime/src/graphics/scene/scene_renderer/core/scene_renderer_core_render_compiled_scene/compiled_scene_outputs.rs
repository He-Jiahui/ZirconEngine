use super::super::super::graph_execution::RenderGraphExecutionRecord;
use super::super::super::mesh::PreparedMeshQueueStats;
use super::super::super::sprite::PreparedSpriteQueueStats;
use super::super::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;
use crate::core::framework::render::RenderCameraTargetGraphImportReport;
use crate::rhi::SubmissionTicket;

/// 编译图提交成功后的单帧交接包，供 SceneRenderer 保存回执和插件只读结果。
/// 构造它之前必须已有真实的 scene submission ticket。
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
#[path = "tests/compiled_scene_outputs.rs"]
mod tests;
