use super::super::super::graph_execution::RenderGraphExecutionRecord;
use super::super::super::mesh::PreparedMeshQueueStats;
use super::super::super::sprite::PreparedSpriteQueueStats;
use super::super::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCompiledSceneOutputs {
    advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
    render_graph_execution: RenderGraphExecutionRecord,
    prepared_mesh_queue_stats: PreparedMeshQueueStats,
    prepared_sprite_queue_stats: PreparedSpriteQueueStats,
}

impl SceneRendererCompiledSceneOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
        render_graph_execution: RenderGraphExecutionRecord,
        prepared_mesh_queue_stats: PreparedMeshQueueStats,
        prepared_sprite_queue_stats: PreparedSpriteQueueStats,
    ) -> Self {
        Self {
            advanced_plugin_readbacks,
            render_graph_execution,
            prepared_mesh_queue_stats,
            prepared_sprite_queue_stats,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_parts(
        self,
    ) -> (
        SceneRendererAdvancedPluginReadbacks,
        RenderGraphExecutionRecord,
        PreparedMeshQueueStats,
        PreparedSpriteQueueStats,
    ) {
        (
            self.advanced_plugin_readbacks,
            self.render_graph_execution,
            self.prepared_mesh_queue_stats,
            self.prepared_sprite_queue_stats,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_scene_outputs_carry_prepared_mesh_queue_stats() {
        let stats = PreparedMeshQueueStats {
            draw_count: 3,
            early_z_draw_count: 2,
            gpu_instancing_candidate_group_count: 1,
            gpu_instancing_candidate_draw_count: 2,
            ..PreparedMeshQueueStats::default()
        };

        let outputs = SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::new(),
            RenderGraphExecutionRecord::default(),
            stats,
            PreparedSpriteQueueStats::default(),
        );

        let (_readbacks, _record, carried_stats, _sprite_stats) = outputs.into_parts();
        assert_eq!(carried_stats, stats);
    }

    #[test]
    fn compiled_scene_outputs_carry_prepared_sprite_queue_stats() {
        let stats = PreparedSpriteQueueStats {
            draw_batch_count: 2,
            sprite_count: 3,
            vertex_count: 18,
            opaque_draw_batch_count: 1,
            transparent_draw_batch_count: 1,
            ..PreparedSpriteQueueStats::default()
        };

        let outputs = SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::new(),
            RenderGraphExecutionRecord::default(),
            PreparedMeshQueueStats::default(),
            stats,
        );

        let (_readbacks, _record, _mesh_stats, carried_stats) = outputs.into_parts();
        assert_eq!(carried_stats, stats);
    }
}
