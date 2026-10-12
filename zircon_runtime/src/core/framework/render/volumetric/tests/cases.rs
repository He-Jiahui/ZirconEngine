use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings,
    RenderPostProcessEffectStackSettings, RenderResolvedPostProcessSettings,
    BUILTIN_POST_PROCESS_VOLUME_COMPONENTS,
};
use crate::core::math::Vec3;

use super::*;

#[test]
fn render_volumetric_froxel_slice_depth_matches_closed_form() {
    let params = FroxelGridParams {
        dimensions: [160, 90, 64],
        near_depth: 0.1,
        far_depth: 1_000.0,
        depth_distribution_exp: 2.0,
    };

    for slice in [0, 1, 31, 63] {
        let normalized = ((slice as Real + 0.5) / 64.0).powf(2.0);
        let expected = 0.1 * (1_000.0_f32 / 0.1).powf(normalized);
        let actual = params.slice_depth(slice);
        assert!((actual - expected).abs() <= expected.max(1.0) * 1.0e-6);
    }
}

#[test]
fn render_volumetric_froxel_slice_depth_matches_all_quality_bounds() {
    for quality in [
        FroxelGridQuality::Low,
        FroxelGridQuality::Medium,
        FroxelGridQuality::High,
    ] {
        let params = FroxelGridParams::for_quality(quality, 0.1, 1_000.0, 2.0);
        let slice_count = params.dimensions[2];
        let depths = (0..slice_count)
            .map(|slice| params.slice_depth(slice))
            .collect::<Vec<_>>();

        assert!(depths
            .iter()
            .all(|depth| *depth >= 0.1 && *depth <= 1_000.0));
        assert!(depths.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            params.slice_depth(slice_count),
            depths[slice_count as usize - 1]
        );
    }
}

#[test]
fn render_volumetric_quality_table_matches_plan18() {
    assert_eq!(FroxelGridQuality::Low.dimensions(), [160, 90, 48]);
    assert_eq!(FroxelGridQuality::Medium.dimensions(), [160, 90, 64]);
    assert_eq!(FroxelGridQuality::High.dimensions(), [160, 90, 96]);
    assert!(!FroxelGridQuality::Low.supports_local_volumes());
    assert!(FroxelGridQuality::Medium.supports_local_volumes());
    assert!(FroxelGridQuality::High.supports_temporal());
    assert_eq!(
        FroxelGridQuality::from_shader_quality(ShaderQualityTier::Low),
        FroxelGridQuality::Low
    );
    assert_eq!(
        FroxelGridQuality::from_shader_quality(ShaderQualityTier::Medium),
        FroxelGridQuality::Medium
    );
    assert_eq!(
        FroxelGridQuality::from_shader_quality(ShaderQualityTier::High),
        FroxelGridQuality::High
    );
    assert_eq!(
        FroxelGridQuality::from_shader_quality(ShaderQualityTier::Ultra),
        FroxelGridQuality::High
    );
}

#[test]
fn render_volumetric_henyey_greenstein_matches_isotropic_and_directional_contracts() {
    let isotropic = henyey_greenstein_phase(0.0, -0.75);
    let expected_isotropic = 1.0 / (4.0 * std::f32::consts::PI);
    assert!((isotropic - expected_isotropic).abs() <= 0.000001);

    let forward = henyey_greenstein_phase(0.6, 1.0);
    let perpendicular = henyey_greenstein_phase(0.6, 0.0);
    let backward = henyey_greenstein_phase(0.6, -1.0);
    assert!(forward > perpendicular);
    assert!(perpendicular > backward);
}

#[test]
fn render_volumetric_hg_phase_normalizes() {
    const SAMPLE_COUNT: usize = 32_768;
    for phase_g in [-0.85, -0.5, 0.0, 0.5, 0.85] {
        let delta_cos = 2.0 / SAMPLE_COUNT as Real;
        let integral = (0..SAMPLE_COUNT)
            .map(|sample| {
                let cos_theta = -1.0 + (sample as Real + 0.5) * delta_cos;
                henyey_greenstein_phase(phase_g, cos_theta)
            })
            .sum::<Real>()
            * delta_cos
            * 2.0
            * std::f32::consts::PI;

        assert!(
            (integral - 1.0).abs() <= 0.002,
            "HG sphere integral for g={phase_g} was {integral}"
        );
    }
}

#[test]
fn render_volumetric_front_to_back_step_matches_homogeneous_closed_form() {
    let step = integrate_volumetric_step(Vec3::splat(2.0), 0.5, 2.0);
    let expected_transmittance = (-1.0_f32).exp();
    let expected_radiance = 2.0 * (1.0 - expected_transmittance) / 0.5;

    assert!((step.transmittance - expected_transmittance).abs() <= 0.000001);
    assert!((step.radiance.x - expected_radiance).abs() <= 0.000001);
    assert_eq!(step.radiance, Vec3::splat(step.radiance.x));

    let vacuum = integrate_volumetric_step(Vec3::splat(3.0), 0.0, 0.25);
    assert_eq!(vacuum.transmittance, 1.0);
    assert_eq!(vacuum.radiance, Vec3::splat(0.75));
}

#[test]
fn render_volumetric_volume_component_uses_shared_volume_evaluator_contract() {
    let descriptor = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
        .iter()
        .find(|descriptor| descriptor.component_id == VOLUMETRIC_FOG_COMPONENT_ID)
        .copied()
        .expect("volumetric fog should be registered in the shared volume registry");
    let mut settings = RenderResolvedPostProcessSettings::new(
        RenderBloomSettings::default(),
        RenderExposureSettings::default(),
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings::default(),
    );

    descriptor
        .apply_values(
            &mut settings,
            &[
                VolumeParamValue::Float(0.08),
                VolumeParamValue::Vec3(Vec3::new(0.2, 0.4, 0.8)),
                VolumeParamValue::Float(0.7),
                VolumeParamValue::Float(0.3),
                VolumeParamValue::Float(2.0),
                VolumeParamValue::Float(3.0),
                VolumeParamValue::Bool(false),
            ],
        )
        .unwrap();

    assert_eq!(
        settings.volumetric_fog,
        VolumetricFogSettings {
            density: 0.08,
            albedo: Vec3::new(0.2, 0.4, 0.8),
            phase_g: 0.7,
            height_falloff: 0.3,
            scattering_intensity: 2.0,
            depth_distribution_exp: 3.0,
            temporal: false,
        }
    );
    assert_eq!(
        descriptor.read_values(&settings),
        vec![
            VolumeParamValue::Float(0.08),
            VolumeParamValue::Vec3(Vec3::new(0.2, 0.4, 0.8)),
            VolumeParamValue::Float(0.7),
            VolumeParamValue::Float(0.3),
            VolumeParamValue::Float(2.0),
            VolumeParamValue::Float(3.0),
            VolumeParamValue::Bool(false),
        ]
    );
}
