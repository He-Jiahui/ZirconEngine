use super::super::{camera::RenderViewportRect, post_process::RenderOutputTransfer};
use crate::core::math::UVec2;

use super::{
    RenderDynamicResolutionController, RenderDynamicResolutionDecision,
    RenderDynamicResolutionDecisionReason, RenderDynamicResolutionScope, RenderPipelinePhase,
    RenderResolutionPolicy, RenderUpscalerKind, RenderViewFamilyPipeline,
};

#[test]
fn temporal_reconstruction_keeps_display_and_history_at_secondary_extent() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(3840, 2160),
        RenderResolutionPolicy::with_temporal_fractions(0.75, 0.5),
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        pipeline.resolution().display_extent(),
        UVec2::new(3840, 2160)
    );
    assert_eq!(
        pipeline.resolution().secondary_extent(),
        UVec2::new(1920, 1080)
    );
    assert_eq!(
        pipeline.resolution().primary_extent(),
        UVec2::new(1440, 810)
    );
    assert_eq!(
        pipeline.resolution().temporal_history_extent(),
        Some(UVec2::new(1920, 1080))
    );
}

#[test]
fn temporal_reconstruction_precedes_hdr_post_process_and_display_mapping() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(2.0 / 3.0, 1.0),
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        pipeline.phases(),
        &[
            RenderPipelinePhase::SceneLinear,
            RenderPipelinePhase::PreReconstructionScenePostProcess,
            RenderPipelinePhase::TemporalReconstruction,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
            RenderPipelinePhase::DisplayMapping,
            RenderPipelinePhase::DisplayPostProcess,
            RenderPipelinePhase::OutputTransform,
            RenderPipelinePhase::Present,
        ]
    );
    assert_eq!(
        pipeline.output_transfer(),
        RenderOutputTransfer::SrgbNonlinear
    );
}

#[test]
fn secondary_spatial_upscale_runs_after_display_mapping() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 0.5),
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        pipeline.phases(),
        &[
            RenderPipelinePhase::SceneLinear,
            RenderPipelinePhase::PreReconstructionScenePostProcess,
            RenderPipelinePhase::TemporalReconstruction,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
            RenderPipelinePhase::DisplayMapping,
            RenderPipelinePhase::DisplayPostProcess,
            RenderPipelinePhase::SecondarySpatialUpscale,
            RenderPipelinePhase::OutputTransform,
            RenderPipelinePhase::Present,
        ]
    );
}

#[test]
fn temporal_history_survives_primary_scale_changes_but_not_secondary_scale_changes() {
    let initial = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Temporal,
    )
    .temporal_history_key()
    .expect("temporal reconstruction owns history");
    let primary_scale_changed = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.75, 1.0),
        RenderUpscalerKind::Temporal,
    )
    .temporal_history_key()
    .expect("temporal reconstruction owns history");
    let secondary_scale_changed = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.75, 0.5),
        RenderUpscalerKind::Temporal,
    )
    .temporal_history_key()
    .expect("temporal reconstruction owns history");

    assert_eq!(initial, primary_scale_changed);
    assert_ne!(initial, secondary_scale_changed);
}

#[test]
fn odd_device_extent_preserves_logical_size_and_pads_only_allocations() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1919, 1079),
        RenderResolutionPolicy::with_scales(0.5, 0.5),
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        pipeline.resolution().secondary_extent(),
        UVec2::new(960, 540)
    );
    assert_eq!(
        pipeline.resolution().secondary_allocation_extent(),
        UVec2::new(960, 544)
    );
    assert_eq!(pipeline.resolution().primary_extent(), UVec2::new(480, 270));
    assert_eq!(
        pipeline.resolution().primary_allocation_extent(),
        UVec2::new(480, 272)
    );
    assert_eq!(
        pipeline.resolution().temporal_history_extent(),
        Some(UVec2::new(960, 544))
    );
}

#[test]
fn temporal_history_identity_includes_the_viewport_rect_and_allocation() {
    let policy = RenderResolutionPolicy::with_scales(0.5, 1.0);
    let left = RenderViewFamilyPipeline::resolve_for_viewport(
        UVec2::new(1920, 1080),
        RenderViewportRect::new(UVec2::ZERO, UVec2::new(960, 1080)),
        policy,
        RenderUpscalerKind::Temporal,
    );
    let right = RenderViewFamilyPipeline::resolve_for_viewport(
        UVec2::new(1920, 1080),
        RenderViewportRect::new(UVec2::new(960, 0), UVec2::new(960, 1080)),
        policy,
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        right.resolution().primary_viewport(),
        RenderViewportRect::new(UVec2::new(480, 0), UVec2::new(480, 540))
    );
    assert_eq!(
        right.resolution().primary_allocation_extent(),
        UVec2::new(960, 544)
    );
    assert_ne!(left.temporal_history_key(), right.temporal_history_key());
    assert!(!right
        .phases()
        .contains(&RenderPipelinePhase::SecondarySpatialUpscale));
}

#[test]
fn non_aligned_viewport_origin_is_scaled_without_allocation_alignment_shift() {
    let pipeline = RenderViewFamilyPipeline::resolve_for_viewport(
        UVec2::new(1919, 1079),
        RenderViewportRect::new(UVec2::new(3, 5), UVec2::new(503, 401)),
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Spatial,
    );

    assert_eq!(
        pipeline.resolution().primary_viewport(),
        RenderViewportRect::new(UVec2::new(1, 2), UVec2::new(252, 201))
    );
    assert_eq!(
        pipeline.resolution().primary_allocation_extent(),
        UVec2::new(256, 208)
    );
}

#[test]
fn viewport_depth_range_survives_clamping_and_resolution_scaling() {
    let pipeline = RenderViewFamilyPipeline::resolve_for_viewport(
        UVec2::new(100, 100),
        RenderViewportRect {
            physical_position: UVec2::new(90, 80),
            physical_size: UVec2::new(20, 30),
            depth_min: 0.2,
            depth_max: 0.8,
        },
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Spatial,
    );

    assert_eq!(
        pipeline.resolution().display_viewport(),
        RenderViewportRect {
            physical_position: UVec2::new(90, 80),
            physical_size: UVec2::new(10, 20),
            depth_min: 0.2,
            depth_max: 0.8,
        }
    );
    assert_eq!(
        pipeline.resolution().primary_viewport(),
        RenderViewportRect {
            physical_position: UVec2::new(45, 40),
            physical_size: UVec2::new(5, 10),
            depth_min: 0.2,
            depth_max: 0.8,
        }
    );
}

#[test]
fn phase_targets_keep_logical_rects_separate_from_padded_allocations() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1919, 1079),
        RenderResolutionPolicy::with_scales(0.5, 0.5),
        RenderUpscalerKind::Temporal,
    );

    let scene_targets = pipeline
        .phase_targets(RenderPipelinePhase::SceneLinear)
        .expect("scene linear phase is always present");
    assert_eq!(scene_targets.input(), None);
    let scene_target = scene_targets.output();
    assert_eq!(scene_target.viewport().physical_size, UVec2::new(480, 270));
    assert_eq!(scene_target.allocation_extent(), UVec2::new(480, 272));

    let pre_reconstruction_targets = pipeline
        .phase_targets(RenderPipelinePhase::PreReconstructionScenePostProcess)
        .expect("pre-reconstruction post process is always present");
    assert_eq!(pre_reconstruction_targets.input(), Some(scene_target));
    assert_eq!(pre_reconstruction_targets.output(), scene_target);

    let temporal_targets = pipeline
        .phase_targets(RenderPipelinePhase::TemporalReconstruction)
        .expect("temporal reconstruction owns a secondary target");
    assert_eq!(temporal_targets.input(), Some(scene_target));
    let temporal_target = temporal_targets.output();
    assert_eq!(
        temporal_target.viewport().physical_size,
        UVec2::new(960, 540)
    );
    assert_eq!(temporal_target.allocation_extent(), UVec2::new(960, 544));

    let post_reconstruction_targets = pipeline
        .phase_targets(RenderPipelinePhase::PostReconstructionScenePostProcess)
        .expect("post-reconstruction post process is always present");
    assert_eq!(post_reconstruction_targets.input(), Some(temporal_target));
    assert_eq!(post_reconstruction_targets.output(), temporal_target);

    let display_post_target = pipeline
        .output_target_for_phase(RenderPipelinePhase::DisplayPostProcess)
        .expect("display post process is always present");
    assert_eq!(display_post_target, temporal_target);

    let spatial_targets = pipeline
        .phase_targets(RenderPipelinePhase::SecondarySpatialUpscale)
        .expect("secondary lowering requires a spatial output transition");
    assert_eq!(spatial_targets.input(), Some(display_post_target));
    let spatial_target = spatial_targets.output();
    assert_eq!(
        spatial_target.viewport().physical_size,
        UVec2::new(1919, 1079)
    );
    assert_eq!(spatial_target.allocation_extent(), UVec2::new(1919, 1079));

    assert_eq!(
        pipeline.output_target_for_phase(RenderPipelinePhase::OutputTransform),
        Some(spatial_target)
    );
}

#[test]
fn spatial_only_phase_targets_keep_post_process_at_primary_resolution() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1919, 1079),
        RenderResolutionPolicy::with_spatial_primary_fraction(0.5),
        RenderUpscalerKind::Spatial,
    );

    let scene_target = pipeline
        .phase_targets(RenderPipelinePhase::SceneLinear)
        .expect("scene linear phase is always present");
    let display_post_target = pipeline
        .phase_targets(RenderPipelinePhase::DisplayPostProcess)
        .expect("display post process is always present");
    assert_eq!(scene_target.input(), None);
    assert_eq!(display_post_target.input(), Some(scene_target.output()));
    let scene_target = scene_target.output();
    let display_post_target = display_post_target.output();
    assert_eq!(scene_target.viewport().physical_size, UVec2::new(960, 540));
    assert_eq!(scene_target.allocation_extent(), UVec2::new(960, 544));
    assert_eq!(display_post_target, scene_target);

    let spatial_targets = pipeline
        .phase_targets(RenderPipelinePhase::PrimarySpatialUpscale)
        .expect("primary lowering requires a spatial output transition");
    assert_eq!(spatial_targets.input(), Some(display_post_target));
    let spatial_target = spatial_targets.output();
    assert_eq!(
        spatial_target.viewport().physical_size,
        UVec2::new(1919, 1079)
    );
    assert_eq!(spatial_target.allocation_extent(), UVec2::new(1919, 1079));
    assert_eq!(
        pipeline.output_target_for_phase(RenderPipelinePhase::OutputTransform),
        Some(spatial_target)
    );
    assert_eq!(
        pipeline.output_target_for_phase(RenderPipelinePhase::Present),
        Some(spatial_target)
    );
}

#[test]
fn spatial_only_pipeline_upscales_after_display_phases() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_spatial_primary_fraction(0.5),
        RenderUpscalerKind::Spatial,
    );

    assert_eq!(
        pipeline.phases(),
        &[
            RenderPipelinePhase::SceneLinear,
            RenderPipelinePhase::PreReconstructionScenePostProcess,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
            RenderPipelinePhase::DisplayMapping,
            RenderPipelinePhase::DisplayPostProcess,
            RenderPipelinePhase::PrimarySpatialUpscale,
            RenderPipelinePhase::OutputTransform,
            RenderPipelinePhase::Present,
        ]
    );
}

#[test]
fn native_resolution_omits_spatial_upscale_and_keeps_present_unpadded() {
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1919, 1079),
        RenderResolutionPolicy::default(),
        RenderUpscalerKind::Spatial,
    );

    let scene_target = pipeline
        .output_target_for_phase(RenderPipelinePhase::SceneLinear)
        .expect("scene linear phase is always present");
    assert_eq!(
        scene_target.viewport().physical_size,
        UVec2::new(1919, 1079)
    );
    assert_eq!(scene_target.allocation_extent(), UVec2::new(1920, 1080));
    assert_eq!(
        pipeline.output_target_for_phase(RenderPipelinePhase::PrimarySpatialUpscale),
        None
    );
    assert_eq!(
        pipeline.output_target_for_phase(RenderPipelinePhase::SecondarySpatialUpscale),
        None
    );

    let present_target = pipeline
        .output_target_for_phase(RenderPipelinePhase::Present)
        .expect("present phase is always present");
    assert_eq!(
        present_target.viewport().physical_size,
        UVec2::new(1919, 1079)
    );
    assert_eq!(present_target.allocation_extent(), UVec2::new(1919, 1079));
    assert_eq!(
        pipeline.output_target_for_phase(RenderPipelinePhase::OutputTransform),
        Some(present_target)
    );
}

#[test]
fn spatial_upscale_phases_cover_primary_secondary_and_dual_paths() {
    let display = UVec2::new(1920, 1080);
    let primary_only = RenderViewFamilyPipeline::resolve(
        display,
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Spatial,
    );
    assert!(primary_only
        .phases()
        .contains(&RenderPipelinePhase::PrimarySpatialUpscale));
    assert!(!primary_only
        .phases()
        .contains(&RenderPipelinePhase::SecondarySpatialUpscale));

    let secondary_only = RenderViewFamilyPipeline::resolve(
        display,
        RenderResolutionPolicy::with_scales(1.0, 0.5),
        RenderUpscalerKind::Spatial,
    );
    assert!(!secondary_only
        .phases()
        .contains(&RenderPipelinePhase::PrimarySpatialUpscale));
    assert!(secondary_only
        .phases()
        .contains(&RenderPipelinePhase::SecondarySpatialUpscale));

    let dual_spatial = RenderViewFamilyPipeline::resolve(
        display,
        RenderResolutionPolicy::with_scales(0.5, 0.5),
        RenderUpscalerKind::Spatial,
    );
    let primary_targets = dual_spatial
        .phase_targets(RenderPipelinePhase::PrimarySpatialUpscale)
        .expect("dual spatial path requires primary-to-secondary upscale");
    let secondary_targets = dual_spatial
        .phase_targets(RenderPipelinePhase::SecondarySpatialUpscale)
        .expect("dual spatial path requires secondary-to-display upscale");
    assert_eq!(primary_targets.output(), secondary_targets.input().unwrap());
    assert_eq!(
        primary_targets.input().unwrap().viewport().physical_size,
        UVec2::new(480, 270)
    );
    assert_eq!(
        primary_targets.output().viewport().physical_size,
        UVec2::new(960, 540)
    );
    assert_eq!(secondary_targets.output().viewport().physical_size, display);

    let temporal_then_secondary = RenderViewFamilyPipeline::resolve(
        display,
        RenderResolutionPolicy::with_scales(0.5, 0.5),
        RenderUpscalerKind::Temporal,
    );
    assert!(!temporal_then_secondary
        .phases()
        .contains(&RenderPipelinePhase::PrimarySpatialUpscale));
    assert!(temporal_then_secondary
        .phases()
        .contains(&RenderPipelinePhase::TemporalReconstruction));
    assert!(temporal_then_secondary
        .phases()
        .contains(&RenderPipelinePhase::SecondarySpatialUpscale));
}

#[test]
fn direct_spatial_policy_defaults_secondary_fraction_to_output_resolution() {
    let policy = RenderResolutionPolicy::with_spatial_primary_fraction(0.5);

    assert_eq!(policy.primary_fraction(), 0.5);
    assert_eq!(policy.secondary_fraction(), 1.0);
}

#[test]
fn dynamic_resolution_decision_is_constructible_by_the_runtime_owner() {
    let scope = RenderDynamicResolutionScope::new(17, 4, RenderUpscalerKind::Temporal);
    let decision = RenderDynamicResolutionDecision::new(
        scope,
        101,
        Some(100),
        0.85,
        1.0,
        RenderDynamicResolutionDecisionReason::CompletedGpuSample,
        false,
    );

    assert_eq!(decision.scope(), scope);
    assert_eq!(decision.decision_generation(), 101);
    assert_eq!(decision.source_frame_generation(), Some(100));
    assert_eq!(decision.primary_fraction(), 0.85);
    assert_eq!(decision.primary_upper_bound(), 1.0);
    assert_eq!(
        decision.reason(),
        RenderDynamicResolutionDecisionReason::CompletedGpuSample
    );
    assert!(!decision.requires_temporal_history_reset());
}

#[test]
fn dynamic_resolution_decision_replaces_only_the_primary_view_family_fraction() {
    let decision = RenderDynamicResolutionDecision::new(
        RenderDynamicResolutionScope::new(17, 4, RenderUpscalerKind::Temporal),
        101,
        Some(100),
        0.5,
        1.0,
        RenderDynamicResolutionDecisionReason::CompletedGpuSample,
        false,
    );
    let pipeline = RenderViewFamilyPipeline::resolve_for_viewport_with_dynamic_resolution_decision(
        UVec2::new(1920, 1080),
        RenderViewportRect::new(UVec2::ZERO, UVec2::new(1920, 1080)),
        RenderResolutionPolicy::with_temporal_fractions(0.75, 0.75),
        RenderUpscalerKind::Temporal,
        decision,
    );

    assert_eq!(
        pipeline.resolution().secondary_extent(),
        UVec2::new(1440, 810)
    );
    assert_eq!(pipeline.resolution().primary_extent(), UVec2::new(720, 405));
    assert_eq!(
        pipeline.resolution().temporal_history_extent(),
        Some(UVec2::new(1440, 816))
    );
}

#[test]
fn dynamic_resolution_decision_normalizes_the_primary_fraction_to_its_upper_bound() {
    let decision = RenderDynamicResolutionDecision::new(
        RenderDynamicResolutionScope::new(17, 4, RenderUpscalerKind::Temporal),
        101,
        Some(100),
        2.0,
        0.5,
        RenderDynamicResolutionDecisionReason::CompletedGpuSample,
        false,
    );

    assert_eq!(decision.primary_fraction(), 0.5);
    assert_eq!(decision.primary_upper_bound(), 0.5);
}

#[test]
fn dynamic_resolution_controller_converges_with_bounded_square_root_feedback() {
    let controller = RenderDynamicResolutionController::new(0.5, 1.0, 16.0, 0.1, 0.25);

    assert_eq!(controller.next_primary_fraction(1.0, 64.0), 0.9);
    assert_eq!(controller.next_primary_fraction(0.5, 4.0), 0.6);
    assert_eq!(controller.next_primary_fraction(0.75, 16.1), 0.75);
    assert_eq!(controller.next_primary_fraction(0.75, f32::NAN), 0.75);
}
