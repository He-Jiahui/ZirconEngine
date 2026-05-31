pub(super) fn compressor_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_gain_db: f32,
    sidechain: Option<&[f32]>,
    gain_state: &mut f32,
) {
    let threshold = db_to_gain(threshold_db);
    let makeup = db_to_gain(makeup_gain_db);
    let frames = buffer.len() / channels;
    let attack = smoothing_coefficient(attack_ms, sample_rate_hz);
    let release = smoothing_coefficient(release_ms, sample_rate_hz);
    let mut gain = if gain_state.is_finite() && *gain_state > 0.0 {
        *gain_state
    } else {
        1.0
    };
    for frame in 0..frames {
        let level = if let Some(sidechain) = sidechain {
            frame_level(sidechain, channels, frame)
        } else {
            frame_level(buffer, channels, frame)
        };
        let target_gain = if level > threshold && threshold > 0.0 {
            let over = level / threshold;
            let compressed = over.powf((1.0 / ratio.max(1.0)) - 1.0);
            compressed.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let smoothing = if target_gain < gain { attack } else { release };
        gain = target_gain + (gain - target_gain) * smoothing;
        let output_gain = gain * makeup;
        for channel in 0..channels {
            buffer[frame * channels + channel] *= output_gain;
        }
    }
    *gain_state = gain;
}

pub(super) fn limit(buffer: &mut [f32], ceiling: f32) {
    let ceiling = ceiling.max(0.0);
    for sample in buffer {
        *sample = sample.clamp(-ceiling, ceiling);
    }
}

fn frame_level(buffer: &[f32], channels: usize, frame: usize) -> f32 {
    let mut level = 0.0_f32;
    for channel in 0..channels {
        level = level.max(
            buffer
                .get(frame * channels + channel)
                .copied()
                .unwrap_or_default()
                .abs(),
        );
    }
    level
}

fn db_to_gain(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

fn smoothing_coefficient(time_ms: f32, sample_rate_hz: u32) -> f32 {
    if time_ms <= 0.0 {
        return 0.0;
    }
    let samples = (time_ms / 1000.0) * sample_rate_hz.max(1) as f32;
    (-1.0 / samples.max(1.0)).exp()
}
