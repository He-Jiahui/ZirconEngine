pub(super) fn pan_stereo(
    buffer: &mut [f32],
    channels: usize,
    pan: f32,
    width: f32,
    left_gain: f32,
    right_gain: f32,
    invert_left_phase: bool,
    invert_right_phase: bool,
) {
    if channels == 0 {
        return;
    }
    let pan = pan.clamp(-1.0, 1.0);
    let left_pan = if pan > 0.0 { 1.0 - pan } else { 1.0 };
    let right_pan = if pan < 0.0 { 1.0 + pan } else { 1.0 };
    let left_phase = if invert_left_phase { -1.0 } else { 1.0 };
    let right_phase = if invert_right_phase { -1.0 } else { 1.0 };
    for frame in 0..(buffer.len() / channels) {
        let left_index = frame * channels;
        let right_index = if channels > 1 {
            frame * channels + 1
        } else {
            left_index
        };
        let left = buffer[left_index];
        let right = buffer[right_index];
        let mid = (left + right) * 0.5;
        let side = (left - right) * 0.5 * width.max(0.0);
        buffer[left_index] = (mid + side) * left_pan * left_gain * left_phase;
        if channels > 1 {
            buffer[right_index] = (mid - side) * right_pan * right_gain * right_phase;
        }
    }
}
