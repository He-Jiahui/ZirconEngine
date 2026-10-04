use crate::graphics::CompiledRenderPipeline;

use super::super::super::post_process::SceneRuntimeFeatureFlags;

pub(crate) fn runtime_features_from_pipeline(
    pipeline: &CompiledRenderPipeline,
) -> SceneRuntimeFeatureFlags {
    let flags = pipeline.runtime_feature_flags();
    SceneRuntimeFeatureFlags {
        deferred_lighting_enabled: flags.deferred_lighting_enabled,
        ssao_enabled: flags.ssao_enabled,
        contact_shadow_enabled: flags.contact_shadow_enabled,
        clustered_lighting_enabled: flags.clustered_lighting_enabled,
        hybrid_global_illumination_enabled: flags.hybrid_global_illumination_enabled,
        temporal_history_enabled: flags.temporal_history_enabled,
        bloom_enabled: flags.bloom_enabled,
        color_grading_enabled: flags.color_grading_enabled,
        anti_alias_enabled: flags.anti_alias_enabled,
        reflection_probes_enabled: flags.reflection_probes_enabled,
        baked_lighting_enabled: flags.baked_lighting_enabled,
        sprite_rendering_enabled: flags.sprite_rendering_enabled,
        particle_rendering_enabled: flags.particle_rendering_enabled,
        virtual_geometry_enabled: flags.virtual_geometry_enabled,
    }
}

#[cfg(test)]
#[path = "tests/runtime_features_from_pipeline.rs"]
mod tests;
