use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureMode, RenderExposureSettings,
    RenderPostProcessEffectStackSettings,
};
use crate::core::math::Vec3;

use super::super::resolved_stack::RenderResolvedPostProcessSettings;
use super::{
    BUILTIN_POST_PROCESS_VOLUME_COMPONENTS, VolumeComponentApplyError, VolumeComponentDescriptor,
    VolumeParamSchema, VolumeParamType, VolumeParamValue, interp_discrete, interp_float_lerp,
    interp_vec3_lerp,
};

const TEST_PARAMS: [VolumeParamSchema; 1] = [VolumeParamSchema::new(
    "value",
    VolumeParamValue::Float(0.0),
    interp_float_lerp,
)];

fn read_test_value(_settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    vec![VolumeParamValue::Float(0.0)]
}

fn apply_test_value(
    _settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    values[0].float(component_id, "value")?;
    Ok(())
}

#[test]
fn render_volume_param_interp_blends_float_vec3_and_discrete_values() {
    assert_eq!(
        interp_float_lerp(
            VolumeParamValue::Float(1.0),
            VolumeParamValue::Float(5.0),
            0.25
        ),
        VolumeParamValue::Float(2.0)
    );
    assert_eq!(
        interp_vec3_lerp(
            VolumeParamValue::Vec3(Vec3::ZERO),
            VolumeParamValue::Vec3(Vec3::new(2.0, 4.0, 6.0)),
            0.5
        ),
        VolumeParamValue::Vec3(Vec3::new(1.0, 2.0, 3.0))
    );
    assert_eq!(
        interp_discrete(VolumeParamValue::Enum(1), VolumeParamValue::Enum(2), 0.49),
        VolumeParamValue::Enum(1)
    );
    assert_eq!(
        interp_discrete(VolumeParamValue::Enum(1), VolumeParamValue::Enum(2), 0.5),
        VolumeParamValue::Enum(2)
    );
}

#[test]
fn render_volume_component_descriptor_applies_defaults_to_resolved_stack() {
    let mut settings = RenderResolvedPostProcessSettings::new(
        RenderBloomSettings {
            intensity: 9.0,
            ..Default::default()
        },
        RenderExposureSettings {
            mode: RenderExposureMode::Histogram,
            compensation_ev: 2.0,
            ..Default::default()
        },
        RenderColorGradingSettings {
            exposure: 2.0,
            ..Default::default()
        },
        RenderPostProcessEffectStackSettings::default(),
    );

    for descriptor in BUILTIN_POST_PROCESS_VOLUME_COMPONENTS {
        descriptor.apply_defaults(&mut settings).unwrap();
    }

    assert_eq!(settings.bloom, RenderBloomSettings::default());
    assert_eq!(settings.exposure, RenderExposureSettings::default());
    assert_eq!(
        settings.color_grading,
        RenderColorGradingSettings::default()
    );
    assert_eq!(
        settings.effect_stack,
        RenderPostProcessEffectStackSettings::default()
    );
}

#[test]
fn render_volume_component_descriptor_applies_authored_values() {
    let mut settings = RenderResolvedPostProcessSettings::new(
        RenderBloomSettings::default(),
        RenderExposureSettings::default(),
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings::default(),
    );
    let descriptor = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
        .iter()
        .find(|descriptor| descriptor.component_id == "post.vignette")
        .copied()
        .unwrap();

    descriptor
        .apply_values(
            &mut settings,
            &[
                VolumeParamValue::Float(0.25),
                VolumeParamValue::Float(0.75),
                VolumeParamValue::Float(0.9),
            ],
        )
        .unwrap();

    assert_eq!(settings.effect_stack.vignette.intensity, 0.25);
    assert_eq!(settings.effect_stack.vignette.smoothness, 0.75);
    assert_eq!(settings.effect_stack.vignette.roundness, 0.9);
}

#[test]
fn render_volume_component_descriptor_applies_exposure_values() {
    let mut settings = RenderResolvedPostProcessSettings::new(
        RenderBloomSettings::default(),
        RenderExposureSettings::default(),
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings::default(),
    );
    let descriptor = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
        .iter()
        .find(|descriptor| descriptor.component_id == "post.exposure")
        .copied()
        .unwrap();

    descriptor
        .apply_values(
            &mut settings,
            &[
                VolumeParamValue::Enum(1),
                VolumeParamValue::Float(7.0),
                VolumeParamValue::Float(1.5),
                VolumeParamValue::Float(-6.0),
                VolumeParamValue::Float(10.0),
                VolumeParamValue::Float(0.2),
                VolumeParamValue::Float(0.8),
                VolumeParamValue::Float(2.0),
                VolumeParamValue::Float(0.5),
            ],
        )
        .unwrap();

    assert_eq!(settings.exposure.mode, RenderExposureMode::Histogram);
    assert_eq!(settings.exposure.manual_ev100, 7.0);
    assert_eq!(settings.exposure.compensation_ev, 1.5);
    assert_eq!(settings.exposure.render_histogram_range(), (-6.0, 10.0));
    assert_eq!(settings.exposure.render_filter_range(), (0.2, 0.8));
    assert_eq!(settings.exposure.render_speed_brighten(), 2.0);
    assert_eq!(settings.exposure.render_speed_darken(), 0.5);
    assert_eq!(
        descriptor.read_values(&settings),
        vec![
            VolumeParamValue::Enum(1),
            VolumeParamValue::Float(7.0),
            VolumeParamValue::Float(1.5),
            VolumeParamValue::Float(-6.0),
            VolumeParamValue::Float(10.0),
            VolumeParamValue::Float(0.2),
            VolumeParamValue::Float(0.8),
            VolumeParamValue::Float(2.0),
            VolumeParamValue::Float(0.5),
        ]
    );
}

#[test]
fn render_volume_component_descriptor_rejects_bad_value_shape() {
    let descriptor = VolumeComponentDescriptor::new(
        "post.test",
        &TEST_PARAMS,
        read_test_value,
        apply_test_value,
    );
    let mut settings = RenderResolvedPostProcessSettings::new(
        RenderBloomSettings::default(),
        RenderExposureSettings::default(),
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings::default(),
    );

    assert_eq!(
        descriptor.apply_values(&mut settings, &[]),
        Err(VolumeComponentApplyError::ParamCountMismatch {
            component_id: "post.test",
            expected: 1,
            actual: 0,
        })
    );
    assert_eq!(
        descriptor.apply_values(&mut settings, &[VolumeParamValue::Uint(1)]),
        Err(VolumeComponentApplyError::ParamTypeMismatch {
            component_id: "post.test",
            param_name: "value",
            expected: VolumeParamType::Float,
            actual: VolumeParamType::Uint,
        })
    );
}
