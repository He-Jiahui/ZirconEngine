use crate::core::framework::render::{
    RenderPipelinePhase, RenderResolutionPolicy, RenderUpscalerKind, RenderViewFamilyPipeline,
};
use crate::core::math::UVec2;

use super::{prepare_upscale, UpscaleExecutionError, UpscaleParamsBufferSlot};

#[test]
fn upscale_preparation_selects_distinct_phase_local_parameter_slots() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 0.75),
        RenderUpscalerKind::Spatial,
    );

    let (primary_slot, primary_params) = prepare_upscale(
        RenderPipelinePhase::PrimarySpatialUpscale,
        pipeline
            .phase_targets(RenderPipelinePhase::PrimarySpatialUpscale)
            .expect("dual spatial pipeline must include the primary upscale phase"),
    )
    .expect("primary upscale preparation");
    let (secondary_slot, secondary_params) = prepare_upscale(
        RenderPipelinePhase::SecondarySpatialUpscale,
        pipeline
            .phase_targets(RenderPipelinePhase::SecondarySpatialUpscale)
            .expect("dual spatial pipeline must include the secondary upscale phase"),
    )
    .expect("secondary upscale preparation");

    assert_eq!(primary_slot, UpscaleParamsBufferSlot::Primary);
    assert_eq!(primary_params.input_output_size, [960, 540, 1440, 810]);
    assert_eq!(secondary_slot, UpscaleParamsBufferSlot::Secondary);
    assert_eq!(secondary_params.input_output_size, [1440, 810, 1920, 1080]);
}

#[test]
fn upscale_preparation_rejects_non_spatial_phase_without_panicking() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(640, 360),
        RenderResolutionPolicy::default(),
        RenderUpscalerKind::Spatial,
    );
    let targets = pipeline
        .phase_targets(RenderPipelinePhase::SceneLinear)
        .expect("scene-linear phase targets");

    assert_eq!(
        prepare_upscale(RenderPipelinePhase::SceneLinear, targets),
        Err(UpscaleExecutionError::InvalidPhase(
            RenderPipelinePhase::SceneLinear
        ))
    );
}

#[test]
fn upscale_preparation_rejects_missing_input_target_without_panicking() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(640, 360),
        RenderResolutionPolicy::default(),
        RenderUpscalerKind::Spatial,
    );
    let scene_targets = pipeline
        .phase_targets(RenderPipelinePhase::SceneLinear)
        .expect("scene-linear phase targets");

    assert_eq!(
        prepare_upscale(RenderPipelinePhase::PrimarySpatialUpscale, scene_targets,),
        Err(UpscaleExecutionError::MissingInputTarget(
            RenderPipelinePhase::PrimarySpatialUpscale
        ))
    );
}
