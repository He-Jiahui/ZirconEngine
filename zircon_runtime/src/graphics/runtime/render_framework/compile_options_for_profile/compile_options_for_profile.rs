use crate::core::framework::render::{
    AdvancedProviderAvailability, RenderCapabilitySummary, RenderQualityProfile,
};

use crate::graphics::RenderPipelineCompileOptions;

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
    use crate::core::framework::render::OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
    use crate::core::framework::render::{
        AdvancedProviderAvailability, RenderCapabilitySummary, RenderQualityProfile,
    };
    use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
    use crate::graphics::{BuiltinRenderFeature, RenderFeatureCapabilityRequirement};

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

    #[test]
    fn hybrid_gi_keeps_the_scene_velocity_producer_without_enabling_taa() {
        let profile = RenderQualityProfile::new("hybrid-gi-without-taa")
            .with_temporal_history(false)
            .with_hybrid_global_illumination(true);
        let options = compile_options_for_profile(
            Some(&profile),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new().with_hybrid_gi_provider("hgi"),
        );

        assert!(options
            .enabled_features
            .contains(&BuiltinRenderFeature::Temporal));
        assert!(!options
            .disabled_features
            .contains(&BuiltinRenderFeature::Temporal));
    }

    #[test]
    fn compile_options_gate_hzb_occlusion_from_backend_capabilities() {
        let supported = compile_options_for_profile(
            None,
            &hzb_occlusion_capabilities(),
            &AdvancedProviderAvailability::new(),
        );
        assert!(supported.enable_hzb_occlusion_culling);

        let unsupported = compile_options_for_profile(
            None,
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new(),
        );
        assert!(!unsupported.enable_hzb_occlusion_culling);

        let insufficient_storage_binding_capacity = compile_options_for_profile(
            None,
            &RenderCapabilitySummary {
                max_storage_buffers_per_shader_stage:
                    HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
                ..hzb_occlusion_capabilities()
            },
            &AdvancedProviderAvailability::new(),
        );
        assert!(!insufficient_storage_binding_capacity.enable_hzb_occlusion_culling);
    }

    #[test]
    fn compile_options_disable_oit_when_fragment_storage_is_unavailable() {
        let supported = compile_options_for_profile(
            None,
            &RenderCapabilitySummary {
                supports_storage_buffers: true,
                supports_fragment_writable_storage: true,
                max_storage_buffers_per_shader_stage: OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                ..RenderCapabilitySummary::default()
            },
            &AdvancedProviderAvailability::new(),
        );
        assert!(!supported.disabled_plugin_features.contains("oit"));

        let unsupported = compile_options_for_profile(
            None,
            &RenderCapabilitySummary {
                supports_storage_buffers: true,
                supports_fragment_writable_storage: false,
                max_storage_buffers_per_shader_stage: OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                ..RenderCapabilitySummary::default()
            },
            &AdvancedProviderAvailability::new(),
        );
        assert!(unsupported.disabled_plugin_features.contains("oit"));
    }

    fn advanced_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            virtual_geometry_supported: true,
            hybrid_global_illumination_supported: true,
            supports_storage_buffers: true,
            max_storage_buffers_per_shader_stage:
                HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            ..RenderCapabilitySummary::default()
        }
    }

    fn hzb_occlusion_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            supports_storage_buffers: true,
            max_storage_buffers_per_shader_stage:
                HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        }
    }
}
