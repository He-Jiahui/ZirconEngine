use zircon_runtime::core::framework::sound::{SoundFilterEffect, SoundFilterMode};

use super::constants::{MAX_Q, MIN_CUTOFF_HZ, MIN_Q, NYQUIST_SAFETY};
use super::shelf::shelf_coefficients;

#[derive(Clone, Copy, Debug)]
pub(super) struct SoundBiquadCoefficients {
    pub(super) b0: f32,
    pub(super) b1: f32,
    pub(super) b2: f32,
    pub(super) a1: f32,
    pub(super) a2: f32,
}

impl SoundBiquadCoefficients {
    pub(super) fn from_filter(filter: SoundFilterEffect, sample_rate_hz: u32) -> Self {
        let sample_rate = sample_rate_hz.max(1) as f32;
        let max_cutoff = (sample_rate * 0.5 * NYQUIST_SAFETY).max(f32::EPSILON);
        let min_cutoff = MIN_CUTOFF_HZ.min(max_cutoff);
        let cutoff = if filter.cutoff_hz.is_finite() {
            filter.cutoff_hz
        } else {
            min_cutoff
        }
        .clamp(min_cutoff, max_cutoff);
        let q = if filter.resonance.is_finite() && filter.resonance > 0.0 {
            filter.resonance.clamp(MIN_Q, MAX_Q)
        } else {
            std::f32::consts::FRAC_1_SQRT_2
        };
        let omega = std::f32::consts::TAU * cutoff / sample_rate;
        let sin = omega.sin();
        let cos = omega.cos();
        let alpha = sin / (2.0 * q);

        let raw = match filter.mode {
            SoundFilterMode::LowPass => RawBiquadCoefficients {
                b0: (1.0 - cos) * 0.5,
                b1: 1.0 - cos,
                b2: (1.0 - cos) * 0.5,
                a0: 1.0 + alpha,
                a1: -2.0 * cos,
                a2: 1.0 - alpha,
            },
            SoundFilterMode::HighPass => RawBiquadCoefficients {
                b0: (1.0 + cos) * 0.5,
                b1: -(1.0 + cos),
                b2: (1.0 + cos) * 0.5,
                a0: 1.0 + alpha,
                a1: -2.0 * cos,
                a2: 1.0 - alpha,
            },
            SoundFilterMode::BandPass => RawBiquadCoefficients {
                b0: alpha,
                b1: 0.0,
                b2: -alpha,
                a0: 1.0 + alpha,
                a1: -2.0 * cos,
                a2: 1.0 - alpha,
            },
            SoundFilterMode::Notch => RawBiquadCoefficients {
                b0: 1.0,
                b1: -2.0 * cos,
                b2: 1.0,
                a0: 1.0 + alpha,
                a1: -2.0 * cos,
                a2: 1.0 - alpha,
            },
            SoundFilterMode::LowShelf => {
                shelf_coefficients(filter.gain_db, filter.resonance, sin, cos, true)
            }
            SoundFilterMode::HighShelf => {
                shelf_coefficients(filter.gain_db, filter.resonance, sin, cos, false)
            }
        };
        raw.normalized()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RawBiquadCoefficients {
    pub(super) b0: f32,
    pub(super) b1: f32,
    pub(super) b2: f32,
    pub(super) a0: f32,
    pub(super) a1: f32,
    pub(super) a2: f32,
}

impl RawBiquadCoefficients {
    fn normalized(self) -> SoundBiquadCoefficients {
        let a0 = if self.a0.abs() > f32::EPSILON {
            self.a0
        } else {
            1.0
        };
        SoundBiquadCoefficients {
            b0: self.b0 / a0,
            b1: self.b1 / a0,
            b2: self.b2 / a0,
            a1: self.a1 / a0,
            a2: self.a2 / a0,
        }
    }
}
