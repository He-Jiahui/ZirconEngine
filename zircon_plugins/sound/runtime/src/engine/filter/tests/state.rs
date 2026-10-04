/// Persistent direct-form biquad history for one effect instance.
#[derive(Clone, Debug, Default)]
pub(crate) struct SoundBiquadFilterState {
    channels: Vec<SoundBiquadChannelState>,
}

impl SoundBiquadFilterState {
    pub(super) fn channel_state(
        &mut self,
        channel: usize,
        channels: usize,
    ) -> &mut SoundBiquadChannelState {
        if self.channels.len() != channels {
            self.channels = vec![SoundBiquadChannelState::default(); channels];
        }
        &mut self.channels[channel]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SoundBiquadChannelState {
    pub(super) x1: f32,
    pub(super) x2: f32,
    pub(super) y1: f32,
    pub(super) y2: f32,
}
