//! 更改质量档位前先验证候选管线与设备能力，成功后视口后续帧才使用新的有效编译选项。
use std::collections::HashMap;

use crate::core::framework::render::{
    RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile, RenderViewportHandle,
};
use crate::graphics::RenderPipelineAsset;

use super::super::capability_validation::{
    validate_compiled_pipeline_capabilities, validate_quality_profile_capabilities,
};
use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn set_quality_profile(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    profile: RenderQualityProfile,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.lock_operation();
    let (capabilities, effective_pipeline, pipeline_asset) = {
        let state = framework.lock_state();
        let active_pipeline = state
            .viewports
            .get(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?
            .pipeline();
        let effective_pipeline = active_pipeline.or(profile.pipeline_override);
        let pipeline_asset = pipeline_asset_for_profile(&state.pipelines, effective_pipeline)?;
        (
            state.stats.capabilities.clone(),
            effective_pipeline,
            pipeline_asset,
        )
    };
    let compiled = pipeline_asset
        .as_ref()
        .map(compile_pipeline_for_validation)
        .transpose()?;
    let profile_name = profile.name.clone();
    let mut state = framework.lock_state();
    if let Some((pipeline, compiled)) = effective_pipeline.zip(compiled.as_ref()) {
        state
            .renderer
            .validate_compiled_pipeline_executors(compiled)
            .map_err(|message| RenderFrameworkError::GraphCompileFailure {
                pipeline: pipeline.raw(),
                message,
            })?;
        validate_compiled_pipeline_capabilities(compiled, &capabilities)?;
    }
    validate_quality_profile_capabilities(effective_pipeline, &profile, &capabilities)?;
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked above");
    record.set_quality_profile(profile);
    state.stats.last_quality_profile = Some(profile_name);
    Ok(())
}

fn pipeline_asset_for_profile(
    pipelines: &HashMap<RenderPipelineHandle, RenderPipelineAsset>,
    pipeline: Option<RenderPipelineHandle>,
) -> Result<Option<RenderPipelineAsset>, RenderFrameworkError> {
    pipeline
        .map(|pipeline| {
            pipelines
                .get(&pipeline)
                .cloned()
                .ok_or(RenderFrameworkError::UnknownPipeline {
                    pipeline: pipeline.raw(),
                })
        })
        .transpose()
}

#[cfg(test)]
#[path = "tests/set_quality_profile.rs"]
mod tests;
