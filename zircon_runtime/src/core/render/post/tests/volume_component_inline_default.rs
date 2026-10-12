use super::*;

const LONG_PLUGIN_PARAMS: [VolumeParamSchema; 12] = [
    float_param("value-0", 0.0),
    float_param("value-1", 1.0),
    float_param("value-2", 2.0),
    float_param("value-3", 3.0),
    float_param("value-4", 4.0),
    float_param("value-5", 5.0),
    float_param("value-6", 6.0),
    float_param("value-7", 7.0),
    float_param("value-8", 8.0),
    float_param("value-9", 9.0),
    float_param("value-10", 10.0),
    float_param("value-11", 11.0),
];

fn read_long_plugin_defaults(
    _settings: &RenderResolvedPostProcessSettings,
) -> Vec<VolumeParamValue> {
    LONG_PLUGIN_PARAMS
        .iter()
        .map(|param| param.default)
        .collect()
}

fn apply_long_plugin_defaults(
    _settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    assert_eq!(component_id, "post.test-long-plugin");
    assert_eq!(values.len(), LONG_PLUGIN_PARAMS.len());
    for (value, param) in values.iter().zip(LONG_PLUGIN_PARAMS) {
        assert_eq!(*value, param.default);
    }
    Ok(())
}

fn default_settings() -> RenderResolvedPostProcessSettings {
    RenderResolvedPostProcessSettings::new(
        RenderBloomSettings::default(),
        RenderExposureSettings::default(),
        RenderColorGradingSettings::default(),
        crate::core::framework::render::RenderPostProcessEffectStackSettings::default(),
    )
}

#[test]
fn render_volume_component_builtin_defaults_fit_inline_capacity() {
    let largest_builtin = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
        .iter()
        .map(|descriptor| descriptor.params.len())
        .max()
        .unwrap();

    assert_eq!(largest_builtin, BUILTIN_VOLUME_PARAM_INLINE_CAPACITY);
}

#[test]
fn render_volume_component_long_plugin_defaults_use_complete_fallback() {
    let descriptor = VolumeComponentDescriptor::new(
        "post.test-long-plugin",
        &LONG_PLUGIN_PARAMS,
        read_long_plugin_defaults,
        apply_long_plugin_defaults,
    );

    descriptor.apply_defaults(&mut default_settings()).unwrap();
}
