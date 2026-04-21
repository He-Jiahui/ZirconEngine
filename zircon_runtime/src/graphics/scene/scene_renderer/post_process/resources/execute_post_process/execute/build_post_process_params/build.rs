use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;

use super::super::super::super::super::post_process_params::PostProcessParams;
use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::baked_lighting::baked_lighting;
use super::color_grading::color_grading;

pub(in super::super) fn build_post_process_params(
    viewport_size: UVec2,
    cluster_dimensions: UVec2,
    extract: &RenderFrameExtract,
    features: SceneRuntimeFeatureFlags,
    history_available: bool,
    reflection_probe_count: u32,
    hybrid_gi_probe_count: u32,
    scheduled_trace_region_count: u32,
) -> PostProcessParams {
    let color_grading = color_grading(extract, features);
    let baked_lighting = baked_lighting(extract, features);

    PostProcessParams {
        viewport_and_clusters: [
            viewport_size.x.max(1),
            viewport_size.y.max(1),
            cluster_dimensions.x.max(1),
            cluster_dimensions.y.max(1),
        ],
        feature_flags: [
            u32::from(features.ssao_enabled),
            u32::from(features.clustered_lighting_enabled),
            u32::from(features.history_resolve_enabled && history_available),
            reflection_probe_count,
        ],
        hybrid_gi_counts: [
            hybrid_gi_probe_count,
            scheduled_trace_region_count,
            u32::from(features.hybrid_global_illumination_enabled && history_available),
            0,
        ],
        blends: [
            0.24,
            0.42,
            0.28,
            if features.bloom_enabled {
                extract.post_process.bloom.intensity.max(0.0)
            } else {
                0.0
            },
        ],
        grading: [
            color_grading.exposure.max(0.0),
            color_grading.contrast.max(0.0),
            color_grading.saturation.max(0.0),
            color_grading.gamma.max(0.001),
        ],
        tint_and_probe: [
            color_grading.tint.x.max(0.0),
            color_grading.tint.y.max(0.0),
            color_grading.tint.z.max(0.0),
            if features.reflection_probes_enabled {
                0.35
            } else {
                0.0
            },
        ],
        hybrid_gi_color_and_intensity: [
            0.32,
            0.38,
            0.46,
            if features.hybrid_global_illumination_enabled && hybrid_gi_probe_count > 0 {
                0.4
            } else {
                0.0
            },
        ],
        baked_color_and_intensity: [
            baked_lighting.color.x.max(0.0),
            baked_lighting.color.y.max(0.0),
            baked_lighting.color.z.max(0.0),
            baked_lighting.intensity.max(0.0),
        ],
    }
}
