use zircon_runtime::core::framework::sound::SoundHrtfProfileDescriptor;

use super::state::SoundHrtfRenderState;

pub(crate) fn apply_loaded_hrtf_profile(
    buffer: &mut [f32],
    channels: usize,
    profile: &SoundHrtfProfileDescriptor,
    state: &mut SoundHrtfRenderState,
) -> bool {
    if channels < 2 || buffer.is_empty() {
        return false;
    }

    let dry = buffer.to_vec();
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        buffer[frame * channels] = convolve_channel_sample(
            &dry,
            state.history(),
            channels,
            frame,
            0,
            profile.left_kernel.as_slice(),
        );
        buffer[frame * channels + 1] = convolve_channel_sample(
            &dry,
            state.history(),
            channels,
            frame,
            1,
            profile.right_kernel.as_slice(),
        );
    }

    state.remember(
        &dry,
        profile
            .left_kernel
            .len()
            .max(profile.right_kernel.len())
            .saturating_sub(1),
        channels,
    );
    true
}

fn convolve_channel_sample(
    dry: &[f32],
    history: &[f32],
    channels: usize,
    frame: usize,
    channel: usize,
    kernel: &[f32],
) -> f32 {
    kernel
        .iter()
        .enumerate()
        .map(|(tap, gain)| sample_with_history(dry, history, channels, frame, channel, tap) * *gain)
        .sum()
}

fn sample_with_history(
    dry: &[f32],
    history: &[f32],
    channels: usize,
    frame: usize,
    channel: usize,
    delay_frames: usize,
) -> f32 {
    if delay_frames == 0 {
        return dry
            .get(frame * channels + channel)
            .copied()
            .unwrap_or_default();
    }
    if frame >= delay_frames {
        return dry[(frame - delay_frames) * channels + channel];
    }

    let frames_before_block = delay_frames - frame;
    let Some(index) = history
        .len()
        .checked_sub(frames_before_block * channels)
        .map(|base| base + channel)
    else {
        return 0.0;
    };
    history.get(index).copied().unwrap_or_default()
}
