use super::super::scene_renderer::SceneRenderer;

pub(in crate::graphics::scene::scene_renderer::core) fn reset_last_runtime_outputs(
    renderer: &mut SceneRenderer,
) {
    renderer.last_render_graph_execution = Default::default();
    renderer.last_prepared_mesh_queue_stats = Default::default();
    renderer.last_prepared_sprite_queue_stats = Default::default();
    renderer.advanced_plugin_outputs.reset();
}
