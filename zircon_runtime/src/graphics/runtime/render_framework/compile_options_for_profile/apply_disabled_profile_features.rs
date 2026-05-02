use crate::core::framework::render::RenderQualityProfile;

use crate::{BuiltinRenderFeature, RenderPipelineCompileOptions};

pub(super) fn apply_disabled_profile_features(
    profile: Option<&RenderQualityProfile>,
    mut options: RenderPipelineCompileOptions,
) -> RenderPipelineCompileOptions {
    if profile.is_some_and(|profile| !profile.features.clustered_lighting) {
        options = options.with_feature_disabled(BuiltinRenderFeature::ClusteredLighting);
    }
    if profile.is_some_and(|profile| !profile.features.screen_space_ambient_occlusion) {
        options = options
            .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
            .with_plugin_feature_disabled("screen_space_ambient_occlusion");
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
        options = options
            .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
            .with_plugin_feature_disabled("reflection_probes");
    }
    if profile.is_some_and(|profile| !profile.features.baked_lighting) {
        options = options
            .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
            .with_plugin_feature_disabled("baked_lighting");
    }
    if profile.is_some_and(|profile| !profile.features.particle_rendering) {
        options = options
            .with_feature_disabled(BuiltinRenderFeature::Particle)
            .with_plugin_feature_disabled("particle");
    }

    options
}
