//! 视口切换管线前先验证候选图、executor、设备能力与质量档位，成功后才改写选择。
use crate::core::framework::render::{
    RenderFrameworkError, RenderPipelineHandle, RenderViewportHandle,
};

use super::super::capability_validation::{
    validate_compiled_pipeline_capabilities, validate_quality_profile_capabilities,
};
use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn set_pipeline_asset(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.lock_operation();
    let (pipeline_asset, capabilities) = {
        let state = framework.lock_state();
        let pipeline_asset = state.pipelines.get(&pipeline).cloned().ok_or(
            RenderFrameworkError::UnknownPipeline {
                pipeline: pipeline.raw(),
            },
        )?;
        (pipeline_asset, state.stats.capabilities.clone())
    };
    let compiled = compile_pipeline_for_validation(&pipeline_asset)?;
    let mut state = framework.lock_state();
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.raw(),
            message,
        })?;
    validate_compiled_pipeline_capabilities(&compiled, &capabilities)?;
    let record =
        state
            .viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
    if let Some(profile) = record.quality_profile() {
        validate_quality_profile_capabilities(Some(pipeline), profile, &capabilities)?;
    }
    record.set_pipeline(pipeline);
    state.stats.last_pipeline = Some(pipeline);
    Ok(())
}

#[cfg(test)]
#[path = "tests/set_pipeline_asset.rs"]
mod tests;
