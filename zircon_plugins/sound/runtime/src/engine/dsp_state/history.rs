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
