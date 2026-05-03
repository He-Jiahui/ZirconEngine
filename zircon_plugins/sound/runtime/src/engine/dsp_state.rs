use zircon_runtime::core::framework::sound::{SoundEffectId, SoundTrackId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SoundEffectStateKey {
    pub(crate) track: SoundTrackId,
    pub(crate) effect: SoundEffectId,
}

impl SoundEffectStateKey {
    pub(crate) fn new(track: SoundTrackId, effect: SoundEffectId) -> Self {
        Self { track, effect }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SoundEffectRuntimeState {
    pub(crate) delay_line: SoundDelayLineState,
    pub(crate) reverb_history: SoundHistoryState,
    pub(crate) convolution_history: SoundHistoryState,
    pub(crate) modulation_history: SoundHistoryState,
    pub(crate) modulated_delay_phase: f32,
    pub(crate) phaser_phase: f32,
    pub(crate) compressor_gain: f32,
}

impl Default for SoundEffectRuntimeState {
    fn default() -> Self {
        Self {
            delay_line: SoundDelayLineState::default(),
            reverb_history: SoundHistoryState::default(),
            convolution_history: SoundHistoryState::default(),
            modulation_history: SoundHistoryState::default(),
            modulated_delay_phase: 0.0,
            phaser_phase: 0.0,
            compressor_gain: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SoundTrackRuntimeState {
    pub(crate) control_delay_line: SoundDelayLineState,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SoundDelayLineState {
    samples: Vec<f32>,
    cursor: usize,
}

impl SoundDelayLineState {
    pub(crate) fn next(&mut self, input: f32, delay_samples: usize, feedback: f32) -> f32 {
        if delay_samples == 0 {
            return input;
        }
        self.ensure_len(delay_samples);
        let delayed = self.samples[self.cursor];
        self.samples[self.cursor] = input;
        self.cursor = (self.cursor + 1) % self.samples.len();
        delayed + input * feedback.clamp(0.0, 0.99)
    }

    fn ensure_len(&mut self, len: usize) {
        if self.samples.len() == len {
            return;
        }
        self.samples = vec![0.0; len];
        self.cursor = 0;
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SoundHistoryState {
    samples: Vec<f32>,
}

impl SoundHistoryState {
    pub(crate) fn sample(
        &self,
        current: &[f32],
        channels: usize,
        frame: usize,
        channel: usize,
        delay_frames: usize,
    ) -> f32 {
        if delay_frames == 0 {
            return current
                .get(frame * channels + channel)
                .copied()
                .unwrap_or_default();
        }
        if frame >= delay_frames {
            return current[(frame - delay_frames) * channels + channel];
        }
        let frames_before_block = delay_frames - frame;
        let Some(index) = self
            .samples
            .len()
            .checked_sub(frames_before_block * channels)
            .map(|base| base + channel)
        else {
            return 0.0;
        };
        self.samples.get(index).copied().unwrap_or_default()
    }

    pub(crate) fn remember(&mut self, current: &[f32], max_frames: usize, channels: usize) {
        let max_samples = max_frames.saturating_mul(channels);
        if max_samples == 0 {
            self.samples.clear();
            return;
        }
        self.samples.extend_from_slice(current);
        if self.samples.len() > max_samples {
            let keep_from = self.samples.len() - max_samples;
            self.samples.drain(0..keep_from);
        }
    }
}
