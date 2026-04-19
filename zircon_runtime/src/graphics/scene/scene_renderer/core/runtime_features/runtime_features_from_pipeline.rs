use crate::{BuiltinRenderFeature, CompiledRenderPipeline};

use super::super::super::post_process::SceneRuntimeFeatureFlags;

pub(crate) fn runtime_features_from_pipeline(
    pipeline: &CompiledRenderPipeline,
) -> SceneRuntimeFeatureFlags {
    let feature_enabled = |feature| {
        pipeline
            .enabled_features
            .iter()
            .any(|enabled| enabled.feature == feature)
    };

    SceneRuntimeFeatureFlags {
        deferred_lighting_enabled: feature_enabled(BuiltinRenderFeature::DeferredLighting)
            && feature_enabled(BuiltinRenderFeature::DeferredGeometry),
        ssao_enabled: feature_enabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion),
        clustered_lighting_enabled: feature_enabled(BuiltinRenderFeature::ClusteredLighting),
        hybrid_global_illumination_enabled: feature_enabled(
            BuiltinRenderFeature::GlobalIllumination,
        ),
        history_resolve_enabled: feature_enabled(BuiltinRenderFeature::HistoryResolve),
        bloom_enabled: feature_enabled(BuiltinRenderFeature::Bloom),
        color_grading_enabled: feature_enabled(BuiltinRenderFeature::ColorGrading),
        reflection_probes_enabled: feature_enabled(BuiltinRenderFeature::ReflectionProbes),
        baked_lighting_enabled: feature_enabled(BuiltinRenderFeature::BakedLighting),
        particle_rendering_enabled: feature_enabled(BuiltinRenderFeature::Particle),
        virtual_geometry_enabled: feature_enabled(BuiltinRenderFeature::VirtualGeometry),
    }
}
