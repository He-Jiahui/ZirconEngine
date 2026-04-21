use crate::core::framework::render::{
    RenderFrameworkError, RenderPipelineHandle, RenderViewportHandle,
};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn set_pipeline_asset(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    if !state.pipelines.contains_key(&pipeline) {
        return Err(RenderFrameworkError::UnknownPipeline {
            pipeline: pipeline.raw(),
        });
    }
    let record =
        state
            .viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
    record.pipeline = Some(pipeline);
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}
