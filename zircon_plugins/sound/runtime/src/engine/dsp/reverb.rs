use super::super::dsp_state::SoundHistoryState;

pub(super) fn reverb_block(
    buffer: &mut [f32],
    channels: usize,
    pre_delay_frames: usize,
    tail_frames: usize,
    damping: f32,
    history: &mut SoundHistoryState,
) {
    let original = buffer.to_vec();
    let taps = [
        pre_delay_frames.max(1),
        tail_frames.max(2) / 2,
        tail_frames.max(3),
    ];
    let max_tap = taps.iter().copied().max().unwrap_or_default();
    for frame in 0..(buffer.len() / channels) {
        for channel in 0..channels {
            let mut wet = 0.0;
            for (tap_index, tap) in taps.iter().copied().enumerate() {
                wet += history.sample(&original, channels, frame, channel, tap)
                    * damping.clamp(0.0, 0.99).powi(tap_index as i32 + 1);
            }
            buffer[frame * channels + channel] += wet;
        }
    }
    history.remember(&original, max_tap, channels);
}

pub(super) fn convolve_block(
    buffer: &mut [f32],
    channels: usize,
    impulse_response: &[f32],
    history: &mut SoundHistoryState,
) {
    if impulse_response.is_empty() {
        return;
    }
    let original = buffer.to_vec();
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        for channel in 0..channels {
            let mut sum = 0.0;
            for (tap, coefficient) in impulse_response.iter().copied().enumerate() {
                sum += history.sample(&original, channels, frame, channel, tap) * coefficient;
            }
            buffer[frame * channels + channel] = sum;
        }
    }
    history.remember(
        &original,
        impulse_response.len().saturating_sub(1),
        channels,
    );
}
