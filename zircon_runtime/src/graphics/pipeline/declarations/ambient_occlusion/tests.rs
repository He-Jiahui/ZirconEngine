use crate::core::framework::render::{AoSourceSettings, RenderPipelineHandle, RenderViewportRect};
use crate::core::math::UVec2;
use crate::graphics::feature::{
    RenderFeatureResourceKind, RenderFeatureResourceVersion, SsaoParams,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

use super::{
    AmbientOcclusionDepthConvention, AmbientOcclusionInputQualification,
    AmbientOcclusionInputSemantic, AmbientOcclusionMethod, AmbientOcclusionOutputs,
    AmbientOcclusionProjectionClass, AoHistoryKey, COMPILED_AO_PROFILE_VERSION, CompiledAoProfile,
    QualifiedAmbientOcclusionInput,
};

#[test]
fn compiled_ao_profile_preserves_physical_source_and_receipt_generation() {
    let profile = CompiledAoProfile::compile(AoSourceSettings::default(), qualification())
        .unwrap()
        .with_pipeline_generation(41);

    assert_eq!(profile.artifact_version(), COMPILED_AO_PROFILE_VERSION);
    assert_eq!(profile.method(), AmbientOcclusionMethod::Gtao);
    assert_eq!(profile.resolution_divisor(), 1);
    assert_eq!(profile.work_plan().slice_count(), 3);
    assert_eq!(profile.work_plan().samples_per_slice_side(), 3);
    assert_eq!(profile.work_plan().sample_count(), 18);
    assert_eq!(profile.pipeline_generation(), 41);
    assert!(
        profile
            .input_qualification()
            .inputs()
            .iter()
            .all(|input| input.producer_generation() == 41)
    );

    let params = SsaoParams::from_compiled_profile(&profile, UVec2::new(128, 64)).unwrap();
    assert_eq!(params.extent_and_sample_counts(), [128, 64, 3, 3]);
    assert_eq!(
        params.world_radius_thickness_bias_falloff(),
        [1.0, 0.15, 0.02, 0.5]
    );
    assert_eq!(params.intensity_and_limits(), [1.0, 0.0, 128.0, 0.03]);
}

#[test]
fn compiled_half_resolution_profile_uses_a_ceil_divided_work_extent() {
    let profile = CompiledAoProfile::compile(
        AoSourceSettings {
            half_resolution: true,
            ..AoSourceSettings::default()
        },
        qualification(),
    )
    .unwrap()
    .with_pipeline_generation(43);

    assert_eq!(profile.resolution_divisor(), 2);
    assert_eq!(profile.work_extent(), UVec2::new(64, 32));

    let params = SsaoParams::from_compiled_profile(&profile, UVec2::new(128, 64)).unwrap();
    assert_eq!(params.extent_and_sample_counts(), [64, 32, 3, 3]);
    assert_eq!(params.input_extent_and_resolution(), [128, 64, 2, 0]);
}

#[test]
fn ssao_params_reject_a_runtime_extent_that_differs_from_the_compiled_receipt() {
    let profile = CompiledAoProfile::compile(AoSourceSettings::default(), qualification())
        .unwrap()
        .with_pipeline_generation(42);

    let error = SsaoParams::from_compiled_profile(&profile, UVec2::new(64, 64)).unwrap_err();

    assert!(error.contains("does not match compiled allocation"));
}

#[test]
fn ambient_occlusion_outputs_reject_a_valid_rect_outside_the_physical_extent() {
    let error = AmbientOcclusionOutputs::new(
        version("ambient-occlusion", "ssao-evaluate"),
        None,
        None,
        None,
        TextureFormat::Rgba8Unorm,
        UVec2::new(64, 64),
        qualification().render_rect(),
        42,
    )
    .unwrap_err();

    assert!(error.contains("valid rect exceeds"));
}

#[test]
fn compiled_ao_profile_rejects_non_physical_or_unqualified_temporal_settings() {
    for (settings, message) in [
        (
            AoSourceSettings {
                radius_meters: 0.0,
                ..AoSourceSettings::default()
            },
            "radius_meters",
        ),
        (
            AoSourceSettings {
                thickness_meters: 2.0,
                ..AoSourceSettings::default()
            },
            "thickness_meters",
        ),
        (
            AoSourceSettings {
                temporal: true,
                ..AoSourceSettings::default()
            },
            "temporal accumulation",
        ),
    ] {
        let error = CompiledAoProfile::compile(settings, qualification()).unwrap_err();
        assert!(error.contains(message), "unexpected error: {error}");
    }
}

#[test]
fn ao_history_key_requires_matching_non_zero_generations() {
    let profile = CompiledAoProfile::compile(AoSourceSettings::default(), qualification())
        .unwrap()
        .with_pipeline_generation(13);
    let outputs = AmbientOcclusionOutputs::new(
        version("ambient-occlusion", "ao-upsample"),
        None,
        None,
        None,
        TextureFormat::R8Unorm,
        UVec2::new(128, 64),
        profile.input_qualification().render_rect(),
        13,
    )
    .unwrap();
    let motion = version("scene-velocity", "motion-vectors");

    assert!(
        AoHistoryKey::new(
            7,
            3,
            RenderPipelineHandle::new(5),
            &profile,
            &motion,
            13,
            &outputs,
        )
        .is_ok()
    );

    let stale = AmbientOcclusionOutputs::new(
        version("ambient-occlusion", "ao-upsample"),
        None,
        None,
        None,
        TextureFormat::R8Unorm,
        UVec2::new(128, 64),
        profile.input_qualification().render_rect(),
        12,
    )
    .unwrap();
    assert!(
        AoHistoryKey::new(
            7,
            3,
            RenderPipelineHandle::new(5),
            &profile,
            &motion,
            13,
            &stale,
        )
        .unwrap_err()
        .contains("does not match")
    );
}

fn qualification() -> AmbientOcclusionInputQualification {
    AmbientOcclusionInputQualification::new(
        AmbientOcclusionProjectionClass::Perspective,
        AmbientOcclusionDepthConvention::StandardZeroToOne,
        RenderViewportRect::new(UVec2::ZERO, UVec2::new(128, 64)),
        UVec2::new(128, 64),
        vec![
            input(
                AmbientOcclusionInputSemantic::StandardDeviceDepth,
                "scene-depth",
                "depth-prepass",
                TextureFormat::Depth32Float,
            ),
            input(
                AmbientOcclusionInputSemantic::WorldNormalSignedUnorm,
                "gbuffer-normal",
                "deferred-geometry",
                TextureFormat::Rgba8Unorm,
            ),
            input(
                AmbientOcclusionInputSemantic::StandardDeviceDepthMaxPyramid,
                "hzb-furthest",
                "hzb-build",
                TextureFormat::Rgba16Float,
            ),
        ],
    )
    .unwrap()
}

fn input(
    semantic: AmbientOcclusionInputSemantic,
    resource_name: &str,
    producer: &str,
    format: TextureFormat,
) -> QualifiedAmbientOcclusionInput {
    QualifiedAmbientOcclusionInput::new(
        semantic,
        version(resource_name, producer),
        TextureDesc::new(resource_name, 128, 64, format, TextureUsage::SAMPLED),
    )
}

fn version(resource_name: &str, producer: &str) -> RenderFeatureResourceVersion {
    RenderFeatureResourceVersion::new(resource_name, RenderFeatureResourceKind::Texture, producer)
}
