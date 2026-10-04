use super::coefficients::RawBiquadCoefficients;
use super::constants::{MAX_GAIN_DB, MAX_SHELF_SLOPE, MIN_GAIN_DB, MIN_SHELF_SLOPE};

pub(super) fn shelf_coefficients(
    gain_db: f32,
    resonance: f32,
    sin: f32,
    cos: f32,
    low_shelf: bool,
) -> RawBiquadCoefficients {
    let gain_db = if gain_db.is_finite() {
        gain_db.clamp(MIN_GAIN_DB, MAX_GAIN_DB)
    } else {
        0.0
    };
    let amplitude = 10.0_f32.powf(gain_db / 40.0);
    let shelf_slope = if resonance.is_finite() && resonance > 0.0 {
        resonance.clamp(MIN_SHELF_SLOPE, MAX_SHELF_SLOPE)
    } else {
        1.0
    };
    let alpha = sin
        * 0.5
        * ((amplitude + amplitude.recip()) * (shelf_slope.recip() - 1.0) + 2.0)
            .max(0.0)
            .sqrt();
    let beta = 2.0 * amplitude.sqrt() * alpha;
    if low_shelf {
        RawBiquadCoefficients {
            b0: amplitude * ((amplitude + 1.0) - (amplitude - 1.0) * cos + beta),
            b1: 2.0 * amplitude * ((amplitude - 1.0) - (amplitude + 1.0) * cos),
            b2: amplitude * ((amplitude + 1.0) - (amplitude - 1.0) * cos - beta),
            a0: (amplitude + 1.0) + (amplitude - 1.0) * cos + beta,
            a1: -2.0 * ((amplitude - 1.0) + (amplitude + 1.0) * cos),
            a2: (amplitude + 1.0) + (amplitude - 1.0) * cos - beta,
        }
    } else {
        RawBiquadCoefficients {
            b0: amplitude * ((amplitude + 1.0) + (amplitude - 1.0) * cos + beta),
            b1: -2.0 * amplitude * ((amplitude - 1.0) + (amplitude + 1.0) * cos),
            b2: amplitude * ((amplitude + 1.0) + (amplitude - 1.0) * cos - beta),
            a0: (amplitude + 1.0) - (amplitude - 1.0) * cos + beta,
            a1: 2.0 * ((amplitude - 1.0) - (amplitude + 1.0) * cos),
            a2: (amplitude + 1.0) - (amplitude - 1.0) * cos - beta,
        }
    }
}
