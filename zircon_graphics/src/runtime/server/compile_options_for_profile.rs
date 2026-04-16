use zircon_render_server::{RenderCapabilitySummary, RenderQualityProfile};

use crate::{BuiltinRenderFeature, RenderPipelineCompileOptions};

pub(in crate::runtime::server) fn compile_options_for_profile(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
) -> RenderPipelineCompileOptions {
    let mut options = RenderPipelineCompileOptions::default().with_async_compute(
        profile.is_none_or(|profile| profile.features.allow_async_compute)
            && capabilities.supports_async_compute,
    );

    if profile.is_some_and(|profile| !profile.features.clustered_lighting) {
        options = options.with_feature_disabled(BuiltinRenderFeature::ClusteredLighting);
    }
    if profile.is_some_and(|profile| !profile.features.screen_space_ambient_occlusion) {
        options = options.with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion);
    }
    if profile.is_some_and(|profile| !profile.features.history_resolve) {
        options = options.with_feature_disabled(BuiltinRenderFeature::HistoryResolve);
    }
    if profile.is_some_and(|profile| !profile.features.bloom) {
        options = options.with_feature_disabled(BuiltinRenderFeature::Bloom);
    }
    if profile.is_some_and(|profile| !profile.features.color_grading) {
        options = options.with_feature_disabled(BuiltinRenderFeature::ColorGrading);
    }
    if profile.is_some_and(|profile| !profile.features.reflection_probes) {
        options = options.with_feature_disabled(BuiltinRenderFeature::ReflectionProbes);
    }
    if profile.is_some_and(|profile| !profile.features.baked_lighting) {
        options = options.with_feature_disabled(BuiltinRenderFeature::BakedLighting);
    }
    if profile.is_some_and(|profile| !profile.features.particle_rendering) {
        options = options.with_feature_disabled(BuiltinRenderFeature::Particle);
    }
    if profile.is_some_and(|profile| profile.features.virtual_geometry)
        && capabilities.virtual_geometry_supported
    {
        options = options.with_feature_enabled(BuiltinRenderFeature::VirtualGeometry);
    }
    if profile.is_some_and(|profile| profile.features.hybrid_global_illumination)
        && capabilities.hybrid_global_illumination_supported
    {
        options = options.with_feature_enabled(BuiltinRenderFeature::GlobalIllumination);
    }

    options
}
