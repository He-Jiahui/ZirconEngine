use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::sound::{
    SoundHrtfProfileDescriptor, SoundListenerId, SoundSourceId,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SoundHrtfRenderStateKey {
    source: SoundSourceId,
    listener: SoundListenerId,
    profile_id: String,
}

impl SoundHrtfRenderStateKey {
    pub(crate) fn new(
        source: SoundSourceId,
        listener: SoundListenerId,
        profile_id: impl Into<String>,
    ) -> Self {
        Self {
            source,
            listener,
            profile_id: profile_id.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SoundHrtfRenderState {
    history: Vec<f32>,
}

impl SoundHrtfRenderState {
    pub(crate) fn has_pending_tail(&self) -> bool {
        self.history.iter().any(|sample| *sample != 0.0)
    }
}

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
            &state.history,
            channels,
            frame,
            0,
            profile.left_kernel.as_slice(),
        );
        buffer[frame * channels + 1] = convolve_channel_sample(
            &dry,
            &state.history,
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

pub(crate) fn prune_hrtf_render_states(
    states: &mut HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
    active_keys: &HashSet<SoundHrtfRenderStateKey>,
) {
    states.retain(|key, _| active_keys.contains(key));
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

impl SoundHrtfRenderState {
    fn remember(&mut self, current: &[f32], max_frames: usize, channels: usize) {
        let max_samples = max_frames.saturating_mul(channels);
        if max_samples == 0 {
            self.history.clear();
            return;
        }
        self.history.extend_from_slice(current);
        if self.history.len() > max_samples {
            let keep_from = self.history.len() - max_samples;
            self.history.drain(0..keep_from);
        }
    }
}
