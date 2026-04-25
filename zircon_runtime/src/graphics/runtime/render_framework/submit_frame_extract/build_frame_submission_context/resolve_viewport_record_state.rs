use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::RenderPipelineAsset;

use super::super::super::capability_validation::validate_quality_profile_capabilities;
use super::super::super::compile_options_for_profile::compile_options_for_profile;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn resolve_viewport_record_state(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<ViewportRecordState, RenderFrameworkError> {
    let state = server.state.lock().unwrap();
    let (
        size,
        pipeline_handle,
        quality_profile,
        previous_visibility,
        previous_hybrid_gi_runtime,
        previous_virtual_geometry_runtime,
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
        let pipeline_handle = record
            .pipeline
            .or_else(|| {
                record
                    .quality_profile
                    .as_ref()
                    .and_then(|profile| profile.pipeline_override)
            })
            .unwrap_or(RenderPipelineAsset::default_forward_plus().handle);
        if let Some(profile) = record.quality_profile.as_ref() {
            validate_quality_profile_capabilities(
                Some(pipeline_handle),
                profile,
                &state.stats.capabilities,
            )?;
        }
        (
            record.descriptor.size,
            pipeline_handle,
            record
                .quality_profile
                .as_ref()
                .map(|profile| profile.name.clone()),
            record
                .history
                .as_ref()
                .map(|history| history.visibility.clone()),
            record.hybrid_gi_runtime.clone(),
            record.virtual_geometry_runtime.clone(),
            compile_options_for_profile(record.quality_profile.as_ref(), &state.stats.capabilities),
            state.stats.capabilities.clone(),
            state.stats.last_generation.unwrap_or(0) + 1,
        )
    };
    let pipeline_asset = state.pipelines.get(&pipeline_handle).cloned().ok_or(
        RenderFrameworkError::UnknownPipeline {
            pipeline: pipeline_handle.raw(),
        },
    )?;

    Ok(ViewportRecordState {
        size,
        pipeline_handle,
        quality_profile,
        previous_visibility,
        previous_hybrid_gi_runtime,
        previous_virtual_geometry_runtime,
        pipeline_asset,
        compile_options,
        capabilities,
        predicted_generation,
    })
}
