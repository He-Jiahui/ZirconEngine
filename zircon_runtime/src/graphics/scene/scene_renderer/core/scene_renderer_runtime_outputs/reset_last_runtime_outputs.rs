use super::super::scene_renderer::SceneRenderer;

pub(in crate::graphics::scene::scene_renderer::core) fn reset_last_runtime_outputs(
    renderer: &mut SceneRenderer,
) {
    renderer.last_render_graph_execution = Default::default();
    renderer.advanced_plugin_outputs.reset();
}
