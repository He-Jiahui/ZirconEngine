use super::{
    RenderScreenSpaceReflectionSettings, MAX_SSR_ROUGHNESS_MIP_BIAS, MIN_SSR_ROUGHNESS_MIP_BIAS,
};

#[test]
fn screen_space_reflection_settings_sanitize_renderer_upload_values() {
    let settings = RenderScreenSpaceReflectionSettings {
        intensity: -0.5,
        thickness: -0.1,
        max_ray_distance: -12.0,
        temporal_blend_factor: 2.0,
        roughness_mip_bias: 5.0,
        ..Default::default()
    };

    assert_eq!(settings.render_intensity(), 0.0);
    assert_eq!(settings.render_thickness(), 0.0);
    assert_eq!(settings.render_max_ray_distance(), 0.0);
    assert_eq!(settings.render_temporal_blend_factor(), 1.0);
    assert_eq!(
        settings.render_roughness_mip_bias(),
        MAX_SSR_ROUGHNESS_MIP_BIAS
    );

    let settings = RenderScreenSpaceReflectionSettings {
        roughness_mip_bias: -5.0,
        ..Default::default()
    };

    assert_eq!(
        settings.render_roughness_mip_bias(),
        MIN_SSR_ROUGHNESS_MIP_BIAS
    );
}
