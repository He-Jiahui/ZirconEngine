pub(in crate::engine::source_environment) fn apply_source_pan(
    buffer: &mut [f32],
    channels: usize,
    pan: f32,
) {
    if channels < 2 || pan == 0.0 {
        return;
    }
    let pan = pan.clamp(-1.0, 1.0);
    let left_gain = if pan > 0.0 { 1.0 - pan } else { 1.0 };
    let right_gain = if pan < 0.0 { 1.0 + pan } else { 1.0 };
    for frame in 0..(buffer.len() / channels) {
        buffer[frame * channels] *= left_gain;
        buffer[frame * channels + 1] *= right_gain;
    }
}
