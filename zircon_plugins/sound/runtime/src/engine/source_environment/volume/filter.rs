pub(in crate::engine::source_environment) fn low_pass_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    cutoff_hz: f32,
    amount: f32,
) {
    if cutoff_hz <= 0.0 || amount <= 0.0 {
        return;
    }
    let dry = buffer.to_vec();
    let rc = 1.0 / (cutoff_hz * std::f32::consts::TAU);
    let dt = 1.0 / sample_rate_hz.max(1) as f32;
    let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
    for channel in 0..channels {
        let mut low = 0.0;
        for frame in 0..(buffer.len() / channels) {
            let index = frame * channels + channel;
            low += alpha * (dry[index] - low);
            buffer[index] = dry[index] * (1.0 - amount) + low * amount;
        }
    }
}
