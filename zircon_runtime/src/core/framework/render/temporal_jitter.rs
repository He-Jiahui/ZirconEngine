use crate::core::math::{Real, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TemporalJitterSample {
    pub offset_pixels: Vec2,
    pub sequence_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalJitterSequence {
    period: u32,
}

impl TemporalJitterSequence {
    pub fn new(period: u32) -> Self {
        Self {
            period: period.max(1),
        }
    }

    pub fn sample(self, frame_index: u64) -> TemporalJitterSample {
        let sequence_index = (frame_index % u64::from(self.period)) as u32 + 1;
        TemporalJitterSample {
            offset_pixels: Vec2::new(
                halton(sequence_index, 2) - 0.5,
                halton(sequence_index, 3) - 0.5,
            ),
            sequence_index,
        }
    }

    pub const fn period(self) -> u32 {
        self.period
    }
}

pub fn halton(mut index: u32, base: u32) -> Real {
    if base < 2 {
        return 0.0;
    }
    let mut factor = 1.0;
    let mut result = 0.0;
    let base = base as Real;
    while index > 0 {
        factor /= base;
        result += factor * (index % base as u32) as Real;
        index /= base as u32;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{TemporalJitterSequence, halton};

    #[test]
    fn render_taa_halton_matches_reference_values() {
        assert_close(halton(1, 2), 0.5);
        assert_close(halton(2, 2), 0.25);
        assert_close(halton(3, 2), 0.75);
        assert_close(halton(1, 3), 1.0 / 3.0);
        assert_close(halton(2, 3), 2.0 / 3.0);
        assert_close(halton(3, 3), 1.0 / 9.0);
    }

    #[test]
    fn render_taa_jitter_sequence_is_periodic_and_avoids_zero_index() {
        let sequence = TemporalJitterSequence::new(8);
        let first = sequence.sample(0);
        let repeated = sequence.sample(8);

        assert_eq!(sequence.period(), 8);
        assert_eq!(first.sequence_index, 1);
        assert_eq!(repeated.sequence_index, 1);
        assert_close(first.offset_pixels.x, 0.0);
        assert_close(first.offset_pixels.y, -1.0 / 6.0);
        assert_eq!(first, repeated);
    }

    #[test]
    fn render_taa_jitter_sequence_clamps_zero_period() {
        let sequence = TemporalJitterSequence::new(0);

        assert_eq!(sequence.period(), 1);
        assert_eq!(sequence.sample(99).sequence_index, 1);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
