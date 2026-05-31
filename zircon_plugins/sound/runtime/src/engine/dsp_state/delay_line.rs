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
