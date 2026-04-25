use crate::core::framework::render::{
    RenderFrameExtract, RenderFrameworkError, RenderPipelineHandle, RenderWorldSnapshotHandle,
};
use crate::scene::world::World;
use crate::{CompiledRenderPipeline, RenderPipelineAsset};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn register_pipeline_asset(
    server: &WgpuRenderFramework,
    pipeline: RenderPipelineAsset,
) -> Result<RenderPipelineHandle, RenderFrameworkError> {
    let handle = pipeline.handle;
    let compiled = compile_pipeline_for_validation(&pipeline)?;

    let mut state = server.state.lock().unwrap();
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: handle.raw(),
            message,
        })?;
    state.pipelines.insert(handle, pipeline);
    state.stats.last_pipeline = Some(handle);
    Ok(handle)
}

pub(in crate::graphics::runtime::render_framework) fn compile_pipeline_for_validation(
    pipeline: &RenderPipelineAsset,
) -> Result<CompiledRenderPipeline, RenderFrameworkError> {
    pipeline.compile(&validation_extract()).map_err(|message| {
        RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.handle.raw(),
            message,
        }
    })
}

fn validation_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(0),
        World::new().to_render_snapshot(),
    )
}
