use super::{
    RenderDepthOfFieldSettings, MAX_DEPTH_OF_FIELD_FOCAL_LENGTH_MM,
    MIN_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT, MIN_DEPTH_OF_FIELD_FOCUS_RANGE,
};

#[test]
fn depth_of_field_lens_settings_are_sanitized_for_renderer_upload() {
    let settings = RenderDepthOfFieldSettings {
        focus_distance: -2.0,
        focus_range: -1.0,
        aperture: -0.5,
        focal_length_mm: 400.0,
        max_blur_radius: -3.0,
        bokeh_blade_count: 2,
        ..Default::default()
    };

    assert_eq!(settings.render_focus_distance(), 0.0);
    assert_eq!(
        settings.render_focus_range(),
        MIN_DEPTH_OF_FIELD_FOCUS_RANGE
    );
    assert_eq!(settings.render_aperture(), 0.0);
    assert_eq!(
        settings.render_focal_length_mm(),
        MAX_DEPTH_OF_FIELD_FOCAL_LENGTH_MM
    );
    assert_eq!(settings.render_max_blur_radius(), 0.0);
    assert_eq!(
        settings.render_bokeh_blade_count(),
        MIN_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT
    );
}
