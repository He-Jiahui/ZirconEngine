use super::super::dsp_state::SoundHistoryState;

pub(super) fn modulated_delay(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    base_delay_frames: usize,
    depth_frames: usize,
    rate_hz: f32,
    feedback: f32,
    history: &mut SoundHistoryState,
    phase: &mut f32,
) {
    let original = buffer.to_vec();
    let frames = buffer.len() / channels;
    let rate_per_frame = rate_hz.max(0.0) / sample_rate_hz.max(1) as f32;
    let max_delay = base_delay_frames.saturating_add(depth_frames);
    for frame in 0..frames {
        let modulation = ((*phase * std::f32::consts::TAU).sin() * 0.5 + 0.5) * depth_frames as f32;
        let delay = base_delay_frames + modulation.round() as usize;
        for channel in 0..channels {
            let delayed = history.sample(&original, channels, frame, channel, delay);
            let index = frame * channels + channel;
            buffer[index] = original[index] + delayed * (0.5 + feedback.clamp(0.0, 0.95) * 0.5);
        }
        *phase = (*phase + rate_per_frame).fract();
    }
    history.remember(&original, max_delay, channels);
}

pub(super) fn phaser_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    rate_hz: f32,
    depth: f32,
    phase_offset: f32,
    phase_state: &mut f32,
) {
    let frames = buffer.len() / channels;
    let rate_per_frame = rate_hz.max(0.0) / sample_rate_hz.max(1) as f32;
    for frame in 0..frames {
        let phase = (*phase_state + phase_offset).fract() * std::f32::consts::TAU;
        let gain = 1.0 - depth.clamp(0.0, 1.0) * (phase.sin() * 0.5 + 0.5);
        for channel in 0..channels {
            buffer[frame * channels + channel] *= gain;
        }
        *phase_state = (*phase_state + rate_per_frame).fract();
    }
}
