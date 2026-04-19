use crate::core::framework::render::{RenderFrameworkError, RenderPipelineHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn reload_pipeline(
    server: &WgpuRenderFramework,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    if !state.pipelines.contains_key(&pipeline) {
        return Err(RenderFrameworkError::UnknownPipeline {
            pipeline: pipeline.raw(),
        });
    }
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}
