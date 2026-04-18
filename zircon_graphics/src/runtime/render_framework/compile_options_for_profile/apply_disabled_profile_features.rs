use zircon_framework::render::RenderQualityProfile;

use crate::{BuiltinRenderFeature, RenderPipelineCompileOptions};

pub(super) fn apply_disabled_profile_features(
    profile: Option<&RenderQualityProfile>,
    mut options: RenderPipelineCompileOptions,
) -> RenderPipelineCompileOptions {
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

    options
}
