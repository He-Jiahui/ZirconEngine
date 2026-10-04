use crate::core::framework::render::{
    RenderPipelinePhase, RenderResolutionPolicy, RenderUpscalerKind, RenderViewFamilyPipeline,
    TaaQualityPreset,
};
use crate::core::math::UVec2;

use super::TaaResolveParams;

#[test]
fn taa_resolve_params_clamp_viewport_and_encode_quality_constants() {
    let params = TaaResolveParams::new(temporal_targets(0.5, 0.75), true, TaaQualityPreset::Medium);

    assert!(params.is_enabled());
    assert_eq!(params.input_viewport, [0, 0, 960, 540]);
    assert_eq!(params.output_viewport, [0, 0, 1440, 810]);
    assert_eq!(params.flags_and_quality[1], TaaQualityPreset::Medium as u32);
    assert_eq!(params.blend_and_clamp[0], 0.9);
    assert!(params.blend_and_clamp[1] > 1.0);
    assert_eq!(params.blend_and_clamp[2], 1.0);
    assert!(params.blend_and_clamp[3] > 0.0);
    assert_eq!(params.responsive_and_reactive[0], 0.07);
    assert!(params.responsive_and_reactive[1] > 1.0);
    assert!(params.responsive_and_reactive[2] < 1.0);
    assert!(params.responsive_and_reactive[3] < 1.0);
}

#[test]
fn taa_resolve_params_map_quality_presets_to_blend_and_rejection() {
    let targets = temporal_targets(0.5, 1.0);
    let low = TaaResolveParams::new(targets, true, TaaQualityPreset::Low);
    let high = TaaResolveParams::new(targets, true, TaaQualityPreset::High);

    assert!(high.blend_and_clamp[0] > low.blend_and_clamp[0]);
    assert!(high.blend_and_clamp[1] < low.blend_and_clamp[1]);
    assert!(high.blend_and_clamp[2] < low.blend_and_clamp[2]);
    assert!(high.blend_and_clamp[3] < low.blend_and_clamp[3]);
    assert!(high.responsive_and_reactive[0] < low.responsive_and_reactive[0]);
    assert!(high.responsive_and_reactive[1] > low.responsive_and_reactive[1]);
    assert!(high.responsive_and_reactive[2] < low.responsive_and_reactive[2]);
    assert!(high.responsive_and_reactive[3] < low.responsive_and_reactive[3]);
}

#[test]
fn taa_resolve_params_disable_history_weight_when_history_is_invalid() {
    let params = TaaResolveParams::new(temporal_targets(0.5, 1.0), false, TaaQualityPreset::High);

    assert!(!params.is_enabled());
    assert_eq!(params.flags_and_quality[1], TaaQualityPreset::High as u32);
    assert_eq!(params.blend_and_clamp[0], 0.94);
    assert_eq!(params.responsive_and_reactive[2], 0.18);
}

fn temporal_targets(
    primary_fraction: f32,
    secondary_fraction: f32,
) -> crate::core::framework::render::RenderViewFamilyPhaseTargets {
    RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_temporal_fractions(primary_fraction, secondary_fraction),
        RenderUpscalerKind::Temporal,
    )
    .phase_targets(RenderPipelinePhase::TemporalReconstruction)
    .expect("temporal view family must include reconstruction")
}
