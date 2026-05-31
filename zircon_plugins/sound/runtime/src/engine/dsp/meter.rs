use zircon_runtime::core::framework::sound::{SoundTrackId, SoundTrackMeter};

pub(crate) fn meter_for(track: SoundTrackId, buffer: &[f32], channels: usize) -> SoundTrackMeter {
    if buffer.is_empty() || channels == 0 {
        return SoundTrackMeter::silent(track);
    }

    let mut peak = [0.0_f32; 2];
    let mut sum = [0.0_f32; 2];
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        let left = buffer[frame * channels].abs();
        let right = if channels > 1 {
            buffer[frame * channels + 1].abs()
        } else {
            left
        };
        peak[0] = peak[0].max(left);
        peak[1] = peak[1].max(right);
        sum[0] += left * left;
        sum[1] += right * right;
    }

    let divisor = frames.max(1) as f32;
    SoundTrackMeter {
        track,
        peak_left: peak[0],
        peak_right: peak[1],
        rms_left: (sum[0] / divisor).sqrt(),
        rms_right: (sum[1] / divisor).sqrt(),
    }
}
