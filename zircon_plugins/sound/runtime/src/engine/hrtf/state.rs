#[derive(Clone, Debug, Default)]
pub(crate) struct SoundHrtfRenderState {
    history: Vec<f32>,
}

impl SoundHrtfRenderState {
    pub(crate) fn has_pending_tail(&self) -> bool {
        self.history.iter().any(|sample| *sample != 0.0)
    }

    pub(super) fn history(&self) -> &[f32] {
        &self.history
    }

    pub(super) fn remember(&mut self, current: &[f32], max_frames: usize, channels: usize) {
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
