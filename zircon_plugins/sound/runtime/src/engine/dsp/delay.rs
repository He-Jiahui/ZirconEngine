use super::super::dsp_state::SoundDelayLineState;

pub(super) fn delay_block(
    buffer: &mut [f32],
    channels: usize,
    delay_frames: usize,
    feedback: f32,
    state: &mut SoundDelayLineState,
) {
    if delay_frames == 0 {
        return;
    }
    let delay_samples = delay_frames.saturating_mul(channels);
    for sample in buffer.iter_mut() {
        *sample = state.next(*sample, delay_samples, feedback);
    }
}
