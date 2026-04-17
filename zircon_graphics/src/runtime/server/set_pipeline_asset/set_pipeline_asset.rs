use zircon_render_server::{RenderPipelineHandle, RenderServerError, RenderViewportHandle};

use super::super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn set_pipeline_asset(
    server: &WgpuRenderServer,
    viewport: RenderViewportHandle,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderServerError> {
    let mut state = server.state.lock().unwrap();
    if !state.pipelines.contains_key(&pipeline) {
        return Err(RenderServerError::UnknownPipeline {
            pipeline: pipeline.raw(),
        });
    }
    let record = state
        .viewports
        .get_mut(&viewport)
        .ok_or(RenderServerError::UnknownViewport {
            viewport: viewport.raw(),
        })?;
    record.pipeline = Some(pipeline);
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}
