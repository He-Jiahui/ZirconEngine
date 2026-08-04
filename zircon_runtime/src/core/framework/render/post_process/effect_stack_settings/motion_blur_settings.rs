use crate::core::math::Real;

const DEFAULT_MOTION_BLUR_SAMPLES: u32 = 1;
const MAX_MOTION_BLUR_SAMPLES: u32 = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMotionBlurSettings {
    pub shutter_angle: Real,
    pub samples: u32,
}

impl Default for RenderMotionBlurSettings {
    fn default() -> Self {
        Self {
            shutter_angle: 0.0,
            samples: DEFAULT_MOTION_BLUR_SAMPLES,
        }
    }
}

impl RenderMotionBlurSettings {
    pub fn is_enabled(self) -> bool {
        self.shutter_angle > 0.0 && self.samples > 0
    }

    pub fn render_shutter_angle(self) -> Real {
        if self.is_enabled() {
            self.shutter_angle.max(0.0)
        } else {
            0.0
        }
    }

    pub fn render_samples(self) -> u32 {
        self.samples.min(MAX_MOTION_BLUR_SAMPLES)
    }
}

#[cfg(test)]
mod tests {
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
}
