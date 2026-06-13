use crate::core::framework::render::{RenderCapabilitySummary, RenderQualityProfile};
use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;

use crate::RenderPipelineCompileOptions;

pub(super) fn new_compile_options(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
) -> RenderPipelineCompileOptions {
    RenderPipelineCompileOptions::default()
        .with_async_compute(
            profile.is_none_or(|profile| profile.features.allow_async_compute)
                && capabilities.supports_async_compute,
        )
        .with_hzb_occlusion_culling(capabilities.hzb_occlusion_culling_supported(
            HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        ))
}
