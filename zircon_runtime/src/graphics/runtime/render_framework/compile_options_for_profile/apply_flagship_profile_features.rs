use crate::core::framework::render::{RenderCapabilitySummary, RenderQualityProfile};

use crate::{RenderFeatureCapabilityRequirement, RenderPipelineCompileOptions};

pub(super) fn apply_flagship_profile_features(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
    mut options: RenderPipelineCompileOptions,
) -> RenderPipelineCompileOptions {
    if profile.is_some_and(|profile| profile.features.virtual_geometry)
        && capabilities.virtual_geometry_supported
    {
        options =
            options.with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
    }
    if profile.is_some_and(|profile| profile.features.hybrid_global_illumination)
        && capabilities.hybrid_global_illumination_supported
    {
        options = options
            .with_capability_enabled(RenderFeatureCapabilityRequirement::HybridGlobalIllumination);
    }

    options
}
