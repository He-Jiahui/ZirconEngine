use crate::graphics::types::GraphicsError;

use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_core_render_compiled_scene::SceneRendererCompiledSceneOutputs;

/// 提交成功后向 SceneRenderer 发布本帧输出；插件读取者通过 take_* 消费各自结果。
pub(in crate::graphics::scene::scene_renderer::core) fn store_last_runtime_outputs(
    renderer: &mut SceneRenderer,
    runtime_outputs: SceneRendererCompiledSceneOutputs,
) -> Result<(), GraphicsError> {
    let (
        advanced_plugin_readbacks,
        render_graph_execution,
        prepared_mesh_queue_stats,
        prepared_sprite_queue_stats,
        output_target_graph_import_report,
    ) = runtime_outputs.into_parts();
    renderer.last_render_graph_execution = render_graph_execution;
    renderer.last_prepared_mesh_queue_stats = prepared_mesh_queue_stats;
    renderer.last_prepared_sprite_queue_stats = prepared_sprite_queue_stats;
    advanced_plugin_readbacks.collect_into_outputs(&mut renderer.advanced_plugin_outputs)?;
    if let Some(report) = output_target_graph_import_report {
        renderer
            .streamer
            .set_last_output_target_graph_import_report(report);
    }

    Ok(())
}
