use crate::core::framework::render::{
    AdvancedProviderAvailability, RenderCapabilitySummary, RenderQualityProfile,
};

use crate::RenderPipelineCompileOptions;

use super::apply_disabled_profile_features::apply_disabled_profile_features;
use super::apply_flagship_profile_features::apply_flagship_profile_features;
use super::new_compile_options::new_compile_options;

pub(in crate::graphics::runtime::render_framework) fn compile_options_for_profile(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
    availability: &AdvancedProviderAvailability,
) -> RenderPipelineCompileOptions {
    let options = new_compile_options(profile, capabilities);
    let options = apply_disabled_profile_features(profile, options);
    apply_flagship_profile_features(profile, capabilities, availability, options)
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        AdvancedProviderAvailability, RenderCapabilitySummary, RenderQualityProfile,
    };
    use crate::RenderFeatureCapabilityRequirement;

    use super::compile_options_for_profile;

    #[test]
    fn compile_options_do_not_enable_advanced_capabilities_without_providers() {
        let profile = RenderQualityProfile::new("advanced")
            .with_virtual_geometry(true)
            .with_hybrid_global_illumination(true);
        let options = compile_options_for_profile(
            Some(&profile),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new(),
        );

        assert!(!options
            .enabled_capabilities
            .contains(&RenderFeatureCapabilityRequirement::VirtualGeometry));
        assert!(!options
            .enabled_capabilities
            .contains(&RenderFeatureCapabilityRequirement::HybridGlobalIllumination));
    }

    #[test]
    fn compile_options_enable_only_provider_backed_advanced_capabilities() {
        let profile = RenderQualityProfile::new("advanced")
            .with_virtual_geometry(true)
            .with_hybrid_global_illumination(true);
        let options = compile_options_for_profile(
            Some(&profile),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new().with_virtual_geometry_provider("vg"),
        );

        assert!(options
            .enabled_capabilities
            .contains(&RenderFeatureCapabilityRequirement::VirtualGeometry));
        assert!(!options
            .enabled_capabilities
            .contains(&RenderFeatureCapabilityRequirement::HybridGlobalIllumination));
    }

    fn advanced_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            virtual_geometry_supported: true,
            hybrid_global_illumination_supported: true,
            supports_storage_buffers: true,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            ..RenderCapabilitySummary::default()
        }
    }
}
