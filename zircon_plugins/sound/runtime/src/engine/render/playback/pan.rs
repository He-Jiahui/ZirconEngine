pub(super) fn playback_channel_gain(pan: f32, channel: usize) -> f32 {
    if channel == 0 {
        if pan > 0.0 {
            1.0 - pan.clamp(0.0, 1.0)
        } else {
            1.0
        }
    } else if pan < 0.0 {
        1.0 + pan.clamp(-1.0, 0.0)
    } else {
        1.0
    }
}
