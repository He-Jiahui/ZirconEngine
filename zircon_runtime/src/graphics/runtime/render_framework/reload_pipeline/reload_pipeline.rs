//! 重载先验证已注册资产，再推进修订并失效缓存；活跃视口还要通过设备能力检查。
use crate::core::framework::render::{RenderFrameworkError, RenderPipelineHandle};
use crate::graphics::RenderPipelineAsset;

use super::super::capability_validation::validate_compiled_pipeline_capabilities;
use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn reload_pipeline(
    framework: &WgpuRenderFramework,
    pipeline: RenderPipelineHandle,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.lock_operation();
    let pipeline_asset =
        {
            let state = framework.lock_state();
            state.pipelines.get(&pipeline).cloned().ok_or(
                RenderFrameworkError::UnknownPipeline {
                    pipeline: pipeline.raw(),
                },
            )?
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
    let default_pipeline = RenderPipelineAsset::DEFAULT_FORWARD_PLUS_HANDLE;
    let active_for_viewport = state
        .viewports
        .values()
        .any(|record| record.effective_pipeline(default_pipeline) == pipeline);
    if active_for_viewport {
        validate_compiled_pipeline_capabilities(&compiled, &state.stats.capabilities)?;
    }
    if let Some(pipeline_asset) = state.pipelines.get_mut(&pipeline) {
        pipeline_asset.bump_revision();
    }
    state.compiled_graph_cache.invalidate_pipeline(pipeline);
    Ok(())
}

#[cfg(test)]
#[path = "tests/reload_pipeline.rs"]
mod tests;
