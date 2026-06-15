use crate::core::framework::render::DEFAULT_CAMERA_EXPOSURE_EV100;
use crate::core::math::Real;

pub const EXPOSURE_HISTOGRAM_BIN_COUNT: u32 = 64;
pub const EXPOSURE_BUFFER_WORD_COUNT: u32 = 4;

const DEFAULT_AUTO_EXPOSURE_MIN_EV100: Real = -8.0;
const DEFAULT_AUTO_EXPOSURE_MAX_EV100: Real = 8.0;
const DEFAULT_AUTO_EXPOSURE_LOW_PERCENT: Real = 0.10;
const DEFAULT_AUTO_EXPOSURE_HIGH_PERCENT: Real = 0.90;
const DEFAULT_AUTO_EXPOSURE_SPEED_BRIGHTEN: Real = 3.0;
const DEFAULT_AUTO_EXPOSURE_SPEED_DARKEN: Real = 1.0;
const MIN_EXPOSURE_RANGE_WIDTH: Real = 0.001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderExposureMode {
    #[default]
    Manual,
    Histogram,
}

impl RenderExposureMode {
    pub const fn from_volume_id(id: u32) -> Self {
        match id {
            1 => Self::Histogram,
            _ => Self::Manual,
        }
    }

    pub const fn volume_id(self) -> u32 {
        match self {
            Self::Manual => 0,
            Self::Histogram => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderExposureSettings {
    pub mode: RenderExposureMode,
    pub manual_ev100: Real,
    pub compensation_ev: Real,
    pub min_ev100: Real,
    pub max_ev100: Real,
    pub low_percent: Real,
    pub high_percent: Real,
    pub speed_brighten: Real,
    pub speed_darken: Real,
}

impl Default for RenderExposureSettings {
    fn default() -> Self {
        Self::manual_ev100(DEFAULT_CAMERA_EXPOSURE_EV100)
    }
}

impl RenderExposureSettings {
    pub const fn manual_ev100(manual_ev100: Real) -> Self {
        Self {
            mode: RenderExposureMode::Manual,
            manual_ev100,
            compensation_ev: 0.0,
            min_ev100: DEFAULT_AUTO_EXPOSURE_MIN_EV100,
            max_ev100: DEFAULT_AUTO_EXPOSURE_MAX_EV100,
            low_percent: DEFAULT_AUTO_EXPOSURE_LOW_PERCENT,
            high_percent: DEFAULT_AUTO_EXPOSURE_HIGH_PERCENT,
            speed_brighten: DEFAULT_AUTO_EXPOSURE_SPEED_BRIGHTEN,
            speed_darken: DEFAULT_AUTO_EXPOSURE_SPEED_DARKEN,
        }
    }

    pub fn histogram() -> Self {
        Self {
            mode: RenderExposureMode::Histogram,
            ..Self::default()
        }
    }

    pub fn render_histogram_range(self) -> (Real, Real) {
        let min_ev100 = self
            .min_ev100
            .min(self.max_ev100 - MIN_EXPOSURE_RANGE_WIDTH);
        let max_ev100 = self.max_ev100.max(min_ev100 + MIN_EXPOSURE_RANGE_WIDTH);
        (min_ev100, max_ev100)
    }

    pub fn render_filter_range(self) -> (Real, Real) {
        let low_percent = self.low_percent.clamp(0.0, 1.0);
        let high_percent = self.high_percent.clamp(low_percent, 1.0);
        (low_percent, high_percent)
    }

    pub fn render_speed_brighten(self) -> Real {
        self.speed_brighten.max(0.0)
    }

    pub fn render_speed_darken(self) -> Real {
        self.speed_darken.max(0.0)
    }
}

#[cfg(test)]
mod tests {
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
}
