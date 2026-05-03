use crate::graphics::types::GraphicsError;

use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_core_render_compiled_scene::SceneRendererCompiledSceneOutputs;

pub(in crate::graphics::scene::scene_renderer::core) fn store_last_runtime_outputs(
    renderer: &mut SceneRenderer,
    runtime_outputs: SceneRendererCompiledSceneOutputs,
) -> Result<(), GraphicsError> {
    let (advanced_plugin_readbacks, render_graph_execution) = runtime_outputs.into_parts();
    renderer.last_render_graph_execution = render_graph_execution;
    advanced_plugin_readbacks.collect_into_outputs(
        &renderer.backend.device,
        &mut renderer.advanced_plugin_outputs,
    )?;

    Ok(())
}
