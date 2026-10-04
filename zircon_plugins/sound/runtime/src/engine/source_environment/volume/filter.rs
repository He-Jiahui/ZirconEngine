//! 区域低通以原地块处理供源环境预览使用；此算法并非 Kira 图内已安装的实时效果。
pub(in crate::engine::source_environment) fn low_pass_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    cutoff_hz: f32,
    amount: f32,
) {
    if channels == 0 || cutoff_hz <= 0.0 || amount <= 0.0 {
        return;
    }
    let rc = 1.0 / (cutoff_hz * std::f32::consts::TAU);
    let dt = 1.0 / sample_rate_hz.max(1) as f32;
    let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
    let frames = buffer.len() / channels;
    for channel in 0..channels {
        let mut low = 0.0;
        for frame in 0..frames {
            let index = frame * channels + channel;
            let dry_sample = buffer[index];
            low += alpha * (dry_sample - low);
            buffer[index] = dry_sample * (1.0 - amount) + low * amount;
        }
    }
}

#[cfg(test)]
#[path = "tests/filter.rs"]
mod tests;
