use super::{
    RenderExposureMode, RenderExposureSettings, EXPOSURE_BUFFER_WORD_COUNT,
    EXPOSURE_HISTOGRAM_BIN_COUNT,
};
use crate::core::framework::render::DEFAULT_CAMERA_EXPOSURE_EV100;

#[test]
fn render_exposure_defaults_to_camera_manual_ev100() {
    let settings = RenderExposureSettings::default();

    assert_eq!(settings.mode, RenderExposureMode::Manual);
    assert_eq!(settings.manual_ev100, DEFAULT_CAMERA_EXPOSURE_EV100);
    assert_eq!(settings.render_histogram_range(), (-8.0, 8.0));
    assert_eq!(settings.render_filter_range(), (0.10, 0.90));
    assert_eq!(EXPOSURE_HISTOGRAM_BIN_COUNT, 64);
    assert_eq!(EXPOSURE_BUFFER_WORD_COUNT, 4);
}

#[test]
fn render_exposure_upload_values_are_sanitized() {
    let settings = RenderExposureSettings {
        min_ev100: 4.0,
        max_ev100: 2.0,
        low_percent: 1.2,
        high_percent: -1.0,
        speed_brighten: -3.0,
        speed_darken: -1.0,
        ..RenderExposureSettings::histogram()
    };

    assert_eq!(settings.render_histogram_range(), (1.999, 2.0));
    assert_eq!(settings.render_filter_range(), (1.0, 1.0));
    assert_eq!(settings.render_speed_brighten(), 0.0);
    assert_eq!(settings.render_speed_darken(), 0.0);
}
