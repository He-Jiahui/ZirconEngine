use zircon_framework::render::{RenderCapabilitySummary, RenderQualityProfile};

use crate::RenderPipelineCompileOptions;

pub(super) fn new_compile_options(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
) -> RenderPipelineCompileOptions {
    RenderPipelineCompileOptions::default().with_async_compute(
        profile.is_none_or(|profile| profile.features.allow_async_compute)
            && capabilities.supports_async_compute,
    )
}
