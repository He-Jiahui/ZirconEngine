use crate::core::framework::render::{
    RenderCapabilitySummary, RenderQualityProfile, DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
    OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
};
use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;

use crate::graphics::RenderPipelineCompileOptions;

pub(super) fn new_compile_options(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
) -> RenderPipelineCompileOptions {
    let mut options = RenderPipelineCompileOptions::default()
        .with_async_compute(
            profile.is_none_or(|profile| profile.features.allow_async_compute)
                && capabilities.supports_async_compute,
        )
        .with_half_resolution_transparency(
            profile.is_some_and(|profile| profile.features.half_resolution_transparency),
        )
        .with_half_resolution_transparency_depth_sigma(
            profile.map_or(DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA, |profile| {
                profile.half_resolution_transparency_depth_sigma
            }),
        )
        .with_hzb_occlusion_culling(capabilities.hzb_occlusion_culling_supported(
            HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        ));
    if !capabilities.oit_supported(OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE) {
        options = options.with_plugin_feature_disabled("oit");
    }
    options
}
