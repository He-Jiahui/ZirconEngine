use zircon_render_server::{RenderPipelineHandle, RenderServerError};

use super::super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn reload_pipeline(
    server: &WgpuRenderServer,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderServerError> {
    let mut state = server.state.lock().unwrap();
    if !state.pipelines.contains_key(&pipeline) {
        return Err(RenderServerError::UnknownPipeline {
            pipeline: pipeline.raw(),
        });
    }
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}
