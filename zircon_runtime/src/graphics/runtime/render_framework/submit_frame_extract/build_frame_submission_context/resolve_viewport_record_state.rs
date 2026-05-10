use crate::core::framework::render::{
    CorePipelineKind, RenderFrameExtract, RenderFrameworkError, RenderPipelineHandle,
    RenderViewportHandle,
};

use crate::RenderPipelineAsset;

use super::super::super::capability_validation::validate_quality_profile_capabilities;
use super::super::super::compile_options_for_profile::compile_options_for_profile;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn resolve_viewport_record_state(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
) -> Result<ViewportRecordState, RenderFrameworkError> {
    let state = server.state.lock().unwrap();
    let (
        size,
        pipeline_handle,
        viewport_generation,
        quality_profile,
        previous_visibility,
        compile_options,
        capabilities,
        predicted_generation,
    ) = {
        let record =
            state
                .viewports
                .get(&viewport)
                .ok_or(RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                })?;
        let pipeline_handle = record.effective_pipeline(default_pipeline_for_extract(extract));
        if let Some(profile) = record.quality_profile() {
            validate_quality_profile_capabilities(
                Some(pipeline_handle),
                profile,
                &state.stats.capabilities,
            )?;
        }
        (
            record.size(),
            pipeline_handle,
            record.generation(),
            record.quality_profile().map(|profile| profile.name.clone()),
            record.history().map(|history| history.visibility().clone()),
            compile_options_for_profile(record.quality_profile(), &state.stats.capabilities),
            state.stats.capabilities.clone(),
            state.stats.last_generation.unwrap_or(0) + 1,
        )
    };
    let pipeline_asset = state.pipelines.get(&pipeline_handle).cloned().ok_or(
        RenderFrameworkError::UnknownPipeline {
            pipeline: pipeline_handle.raw(),
        },
    )?;

    Ok(ViewportRecordState::new(
        size,
        pipeline_handle,
        viewport_generation,
        quality_profile,
        previous_visibility,
        pipeline_asset,
        compile_options,
        capabilities,
        predicted_generation,
    ))
}

fn default_pipeline_for_extract(extract: &RenderFrameExtract) -> RenderPipelineHandle {
    match extract.view.core_pipeline {
        CorePipelineKind::Core2d => RenderPipelineAsset::default_core2d().handle,
        CorePipelineKind::Core3d => RenderPipelineAsset::default_forward_plus().handle,
    }
}
