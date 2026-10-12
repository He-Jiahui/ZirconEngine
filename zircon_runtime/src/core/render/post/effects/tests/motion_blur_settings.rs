use super::{RenderMotionBlurSettings, MAX_MOTION_BLUR_SAMPLES};

#[test]
fn motion_blur_settings_require_shutter_and_samples_and_clamp_upload_values() {
    assert!(!RenderMotionBlurSettings::default().is_enabled());
    assert_eq!(
        RenderMotionBlurSettings::default().render_shutter_angle(),
        0.0
    );
    assert!(!RenderMotionBlurSettings {
        shutter_angle: 0.5,
        samples: 0,
    }
    .is_enabled());
    assert_eq!(
        RenderMotionBlurSettings {
            shutter_angle: 0.5,
            samples: 0,
        }
        .render_shutter_angle(),
        0.0
    );

    let settings = RenderMotionBlurSettings {
        shutter_angle: 0.5,
        samples: MAX_MOTION_BLUR_SAMPLES + 8,
    };

    assert!(settings.is_enabled());
    assert_eq!(settings.render_shutter_angle(), 0.5);
    assert_eq!(settings.render_samples(), MAX_MOTION_BLUR_SAMPLES);
}
