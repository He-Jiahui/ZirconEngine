use zircon_runtime::core::framework::sound::SoundFilterEffect;

use super::coefficients::SoundBiquadCoefficients;
use super::state::SoundBiquadFilterState;

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
