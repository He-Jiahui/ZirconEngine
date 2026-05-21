use zircon_runtime::core::framework::sound::{SoundFilterEffect, SoundFilterMode};

const MIN_CUTOFF_HZ: f32 = 5.0;
const NYQUIST_SAFETY: f32 = 0.9;
const MIN_Q: f32 = 0.05;
const MAX_Q: f32 = 24.0;
const MIN_SHELF_SLOPE: f32 = 0.05;
const MAX_SHELF_SLOPE: f32 = 4.0;
const MIN_GAIN_DB: f32 = -48.0;
const MAX_GAIN_DB: f32 = 48.0;

/// Persistent direct-form biquad history for one effect instance.
#[derive(Clone, Debug, Default)]
pub(crate) struct SoundBiquadFilterState {
    channels: Vec<SoundBiquadChannelState>,
}

impl SoundBiquadFilterState {
    fn channel_state(&mut self, channel: usize, channels: usize) -> &mut SoundBiquadChannelState {
        if self.channels.len() != channels {
            self.channels = vec![SoundBiquadChannelState::default(); channels];
        }
        &mut self.channels[channel]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SoundBiquadChannelState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

pub(crate) fn apply_biquad_filter_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    filter: SoundFilterEffect,
    state: &mut SoundBiquadFilterState,
) {
    if buffer.is_empty() || channels == 0 {
        return;
    }

    let coefficients = SoundBiquadCoefficients::from_filter(filter, sample_rate_hz);
    let frames = buffer.len() / channels;
    for channel in 0..channels {
        let channel_state = state.channel_state(channel, channels);
        for frame in 0..frames {
            let index = frame * channels + channel;
            let input = buffer[index];
            let output = coefficients.b0 * input
                + coefficients.b1 * channel_state.x1
                + coefficients.b2 * channel_state.x2
                - coefficients.a1 * channel_state.y1
                - coefficients.a2 * channel_state.y2;
            channel_state.x2 = channel_state.x1;
            channel_state.x1 = input;
            channel_state.y2 = channel_state.y1;
            channel_state.y1 = output;
            buffer[index] = output;
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SoundBiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl SoundBiquadCoefficients {
    fn from_filter(filter: SoundFilterEffect, sample_rate_hz: u32) -> Self {
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
struct RawBiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a0: f32,
    a1: f32,
    a2: f32,
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

fn shelf_coefficients(
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
