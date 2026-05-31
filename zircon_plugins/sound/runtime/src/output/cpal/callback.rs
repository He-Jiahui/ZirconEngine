use std::sync::atomic::Ordering;

use super::shared_state::CpalOutputSharedState;

pub(in crate::output::cpal) fn drain_callback_output(
    shared: &CpalOutputSharedState,
    output: &mut [f32],
) {
    let sequence_index = shared.callback_count.fetch_add(1, Ordering::Relaxed);
    shared
        .last_callback_sequence
        .store(sequence_index, Ordering::Relaxed);
    let underrun = match shared.ring_buffer.lock() {
        Ok(mut buffer) => buffer.drain_into_with_silence(output),
        Err(_) => {
            output.fill(0.0);
            output.len()
        }
    };
    if underrun > 0 {
        shared
            .underrun_count
            .fetch_add(underrun as u64, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::super::super::ring_buffer::SoundOutputRingBuffer;
    use super::super::shared_state::CpalOutputSharedState;
    use super::drain_callback_output;

    #[test]
    fn output_device_cpal_callback_drain_zero_fills_underrun_and_counts_callback() {
        let shared = CpalOutputSharedState::new(SoundOutputRingBuffer::new(2));
        shared.ring_buffer.lock().unwrap().push_samples(&[0.25]);

        let mut output = [9.0, 9.0, 9.0];
        drain_callback_output(&shared, &mut output);

        assert_eq!(output, [0.25, 0.0, 0.0]);
        assert_eq!(shared.callback_count.load(Ordering::Relaxed), 1);
        assert_eq!(shared.underrun_count.load(Ordering::Relaxed), 2);
    }
}
