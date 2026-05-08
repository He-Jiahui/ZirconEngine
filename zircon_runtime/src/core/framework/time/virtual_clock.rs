use std::time::Duration;

/// Game-time marker. Virtual time may be paused, scaled, and clamped per update.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Virtual {
    max_delta: Duration,
    paused: bool,
    relative_speed: f64,
    effective_speed: f64,
}

impl Default for Virtual {
    fn default() -> Self {
        Self {
            max_delta: Duration::from_millis(250),
            paused: false,
            relative_speed: 1.0,
            effective_speed: 1.0,
        }
    }
}

impl Virtual {
    pub fn max_delta(&self) -> Duration {
        self.max_delta
    }

    pub fn set_max_delta(&mut self, max_delta: Duration) {
        assert_ne!(max_delta, Duration::ZERO, "max delta must be non-zero");
        self.max_delta = max_delta;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn unpause(&mut self) {
        self.paused = false;
    }

    pub fn relative_speed_f64(&self) -> f64 {
        self.relative_speed
    }

    pub fn set_relative_speed_f64(&mut self, speed: f64) {
        assert!(speed.is_finite(), "relative speed must be finite");
        assert!(
            speed.is_sign_positive() || speed == 0.0,
            "relative speed must be non-negative"
        );
        self.relative_speed = speed;
    }

    pub fn effective_speed_f64(&self) -> f64 {
        self.effective_speed
    }

    pub(crate) fn effective_speed_for_next_delta(&mut self) -> f64 {
        self.effective_speed = if self.paused {
            0.0
        } else {
            self.relative_speed
        };
        self.effective_speed
    }
}
