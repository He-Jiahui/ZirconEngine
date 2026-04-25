use crate::core::framework::render::{RenderFrameworkError, RenderPipelineHandle};

use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn reload_pipeline(
    server: &WgpuRenderFramework,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    let pipeline_asset =
        state
            .pipelines
            .get(&pipeline)
            .cloned()
            .ok_or(RenderFrameworkError::UnknownPipeline {
                pipeline: pipeline.raw(),
            })?;
    let compiled = compile_pipeline_for_validation(&pipeline_asset)?;
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.raw(),
            message,
        })?;
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}
