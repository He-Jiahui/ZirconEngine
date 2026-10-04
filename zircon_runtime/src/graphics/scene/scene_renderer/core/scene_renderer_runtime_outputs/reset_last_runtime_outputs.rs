use super::super::scene_renderer::SceneRenderer;

/// 新帧开始时清空旧提交回执和插件结果，防止失败帧向 Runtime 暴露上一帧输出。
pub(in crate::graphics::scene::scene_renderer::core) fn reset_last_runtime_outputs(
    renderer: &mut SceneRenderer,
) {
    renderer.last_render_graph_execution = Default::default();
    renderer.last_prepared_mesh_queue_stats = Default::default();
    renderer.last_prepared_sprite_queue_stats = Default::default();
    renderer.last_frame_submission_receipt = None;
    renderer.advanced_plugin_outputs.reset();
}
