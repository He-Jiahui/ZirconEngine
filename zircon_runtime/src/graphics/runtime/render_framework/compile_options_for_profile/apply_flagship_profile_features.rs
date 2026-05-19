use crate::core::framework::render::{
    AdvancedProviderAvailability, RenderCapabilitySummary, RenderQualityProfile,
};

use crate::{RenderFeatureCapabilityRequirement, RenderPipelineCompileOptions};

pub(super) fn apply_flagship_profile_features(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
    availability: &AdvancedProviderAvailability,
    mut options: RenderPipelineCompileOptions,
) -> RenderPipelineCompileOptions {
    if profile.is_some_and(|profile| profile.features.virtual_geometry)
        && capabilities.virtual_geometry_supported
        && availability.virtual_geometry_provider_id.is_some()
    {
        options =
            options.with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
    }
    if profile.is_some_and(|profile| profile.features.hybrid_global_illumination)
        && capabilities.hybrid_global_illumination_supported
        && availability.hybrid_gi_provider_id.is_some()
    {
        options = options
            .with_capability_enabled(RenderFeatureCapabilityRequirement::HybridGlobalIllumination);
    }

    options
}
