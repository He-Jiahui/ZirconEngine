use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RendererFeatureAsset,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CompiledRenderPipelineRuntimeFeatureFlags {
    pub(crate) deferred_lighting_enabled: bool,
    pub(crate) ssao_enabled: bool,
    pub(crate) contact_shadow_enabled: bool,
    pub(crate) clustered_lighting_enabled: bool,
    pub(crate) hybrid_global_illumination_enabled: bool,
    pub(crate) temporal_history_enabled: bool,
    pub(crate) bloom_enabled: bool,
    pub(crate) color_grading_enabled: bool,
    pub(crate) anti_alias_enabled: bool,
    pub(crate) screen_space_anti_alias_capability_enabled: bool,
    pub(crate) reflection_probes_enabled: bool,
    pub(crate) baked_lighting_enabled: bool,
    pub(crate) sprite_rendering_enabled: bool,
    pub(crate) particle_rendering_enabled: bool,
    pub(crate) virtual_geometry_enabled: bool,
}

impl CompiledRenderPipelineRuntimeFeatureFlags {
    pub(super) fn from_compiled_inputs(
        enabled_features: &[RendererFeatureAsset],
        capability_requirements: &[RenderFeatureCapabilityRequirement],
    ) -> Self {
        let mut flags = Self::default();
        let mut deferred_geometry_enabled = false;
        let mut deferred_lighting_enabled = false;
        for feature in enabled_features {
            match feature.builtin_feature() {
                Some(BuiltinRenderFeature::DeferredGeometry) => deferred_geometry_enabled = true,
                Some(BuiltinRenderFeature::DeferredLighting) => deferred_lighting_enabled = true,
                Some(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion) => {
                    flags.ssao_enabled = true;
                }
                Some(BuiltinRenderFeature::ClusteredLighting) => {
                    flags.clustered_lighting_enabled = true;
                }
                Some(BuiltinRenderFeature::Temporal) => flags.temporal_history_enabled = true,
                Some(BuiltinRenderFeature::Bloom) => flags.bloom_enabled = true,
                Some(BuiltinRenderFeature::ColorGrading) => flags.color_grading_enabled = true,
                Some(BuiltinRenderFeature::AntiAlias) => flags.anti_alias_enabled = true,
                Some(BuiltinRenderFeature::Sprite) => flags.sprite_rendering_enabled = true,
                Some(_) => {}
                None => match feature.feature_name().as_str() {
                    "screen_space_ambient_occlusion" => flags.ssao_enabled = true,
                    "contact_shadow" => flags.contact_shadow_enabled = true,
                    "reflection_probes" => flags.reflection_probes_enabled = true,
                    "baked_lighting" => flags.baked_lighting_enabled = true,
                    "particle" => flags.particle_rendering_enabled = true,
                    _ => {}
                },
            }
            flags.hybrid_global_illumination_enabled |= feature
                .requires_capability(RenderFeatureCapabilityRequirement::HybridGlobalIllumination);
            flags.virtual_geometry_enabled |=
                feature.requires_capability(RenderFeatureCapabilityRequirement::VirtualGeometry);
        }
        flags.deferred_lighting_enabled = deferred_geometry_enabled && deferred_lighting_enabled;
        flags.screen_space_anti_alias_capability_enabled = capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias);
        flags
    }
}
