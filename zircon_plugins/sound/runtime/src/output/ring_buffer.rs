use std::collections::VecDeque;

#[derive(Debug)]
pub(crate) struct SoundOutputRingBuffer {
    samples: VecDeque<f32>,
    capacity_samples: usize,
}

impl SoundOutputRingBuffer {
    pub(crate) fn new(capacity_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity_samples.max(1)),
            capacity_samples: capacity_samples.max(1),
        }
    }

    pub(crate) fn available_samples(&self) -> usize {
        self.samples.len()
    }

    pub(crate) fn capacity_samples(&self) -> usize {
        self.capacity_samples
    }

    pub(crate) fn push_samples(&mut self, samples: &[f32]) -> usize {
        let dropped = self
            .samples
            .len()
            .saturating_add(samples.len())
            .saturating_sub(self.capacity_samples);
        let mut remaining_drops = dropped;
        while remaining_drops > 0 && !self.samples.is_empty() {
            self.samples.pop_front();
            remaining_drops -= 1;
        }
        self.samples
            .extend(samples.iter().skip(remaining_drops).copied());
        dropped
    }

    pub(crate) fn drain_into_with_silence(&mut self, output: &mut [f32]) -> usize {
        let mut underrun_samples = 0;
        for sample in output {
            if let Some(buffered) = self.samples.pop_front() {
                *sample = buffered;
            } else {
                *sample = 0.0;
                underrun_samples += 1;
            }
        }
        underrun_samples
    }
}

#[cfg(test)]
mod tests {
    use super::SoundOutputRingBuffer;

    #[test]
    fn ring_buffer_preserves_fifo_order_across_partial_reads() {
        let mut buffer = SoundOutputRingBuffer::new(8);
        assert_eq!(buffer.capacity_samples(), 8);
        assert_eq!(buffer.push_samples(&[0.1, 0.2, 0.3, 0.4]), 0);
        assert_eq!(buffer.available_samples(), 4);

        let mut first = [0.0; 2];
        assert_eq!(buffer.drain_into_with_silence(&mut first), 0);
        assert_eq!(first, [0.1, 0.2]);

        assert_eq!(buffer.push_samples(&[0.5, 0.6]), 0);
        let mut second = [0.0; 4];
        assert_eq!(buffer.drain_into_with_silence(&mut second), 0);
        assert_eq!(second, [0.3, 0.4, 0.5, 0.6]);
    }

    #[test]
    fn ring_buffer_zero_fills_shortage_and_preserves_future_reads() {
        let mut buffer = SoundOutputRingBuffer::new(4);
        assert_eq!(buffer.push_samples(&[0.25]), 0);
        let mut first = [9.0; 3];
        assert_eq!(buffer.drain_into_with_silence(&mut first), 2);
        assert_eq!(first, [0.25, 0.0, 0.0]);

        assert_eq!(buffer.push_samples(&[0.5, 0.75]), 0);
        let mut second = [0.0; 2];
        assert_eq!(buffer.drain_into_with_silence(&mut second), 0);
        assert_eq!(second, [0.5, 0.75]);
    }

    #[test]
    fn ring_buffer_drops_oldest_samples_on_overflow() {
        let mut buffer = SoundOutputRingBuffer::new(3);
        assert_eq!(buffer.push_samples(&[1.0, 2.0, 3.0]), 0);
        assert_eq!(buffer.push_samples(&[4.0, 5.0]), 2);
        let mut output = [0.0; 3];
        assert_eq!(buffer.drain_into_with_silence(&mut output), 0);
        assert_eq!(output, [3.0, 4.0, 5.0]);
    }

    #[test]
    fn ring_buffer_oversized_push_keeps_newest_capacity_window() {
        let mut buffer = SoundOutputRingBuffer::new(3);
        assert_eq!(buffer.push_samples(&[1.0, 2.0, 3.0, 4.0]), 1);
        let mut output = [0.0; 3];
        assert_eq!(buffer.drain_into_with_silence(&mut output), 0);
        assert_eq!(output, [2.0, 3.0, 4.0]);
    }
}
