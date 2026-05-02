use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectKind, SoundFilterMode, SoundImpulseResponseId,
    SoundTrackControls, SoundTrackId, SoundTrackMeter,
};

pub(crate) fn apply_track_effects(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    effects: &[SoundEffectDescriptor],
    pre_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    post_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
) {
    for effect in effects {
        if !effect.enabled || effect.bypass {
            continue;
        }
        let dry = buffer.to_vec();
        match &effect.kind {
            SoundEffectKind::Gain(gain) => multiply(buffer, gain.gain),
            SoundEffectKind::Filter(filter) => filter_block(
                buffer,
                channels,
                sample_rate_hz,
                filter.mode,
                filter.cutoff_hz,
            ),
            SoundEffectKind::Reverb(reverb) => reverb_block(
                buffer,
                channels,
                reverb.pre_delay_frames,
                reverb.tail_frames,
                reverb.damping,
            ),
            SoundEffectKind::ConvolutionReverb(convolution) => {
                if let Some(ir) = impulse_responses.get(&convolution.impulse_response) {
                    convolve_block(buffer, channels, ir);
                } else if convolution.fallback_to_algorithmic {
                    reverb_block(
                        buffer,
                        channels,
                        convolution.latency_frames,
                        convolution.latency_frames.max(8),
                        0.35,
                    );
                }
            }
            SoundEffectKind::Compressor(compressor) => {
                let sidechain = compressor.sidechain.and_then(|sidechain| {
                    if sidechain.pre_effects {
                        pre_effect_sidechain_buffers
                            .get(&sidechain.track)
                            .map(Vec::as_slice)
                    } else {
                        post_effect_sidechain_buffers
                            .get(&sidechain.track)
                            .map(Vec::as_slice)
                    }
                });
                compressor_block(
                    buffer,
                    channels,
                    compressor.threshold_db,
                    compressor.ratio,
                    compressor.makeup_gain_db,
                    sidechain,
                );
            }
            SoundEffectKind::WaveShaper(shaper) => waveshape(buffer, shaper.drive),
            SoundEffectKind::Flanger(flanger) => modulated_delay(
                buffer,
                channels,
                sample_rate_hz,
                flanger.delay_frames,
                flanger.depth_frames,
                flanger.rate_hz,
                flanger.feedback,
            ),
            SoundEffectKind::Phaser(phaser) => phaser_block(
                buffer,
                channels,
                sample_rate_hz,
                phaser.rate_hz,
                phaser.depth,
                phaser.phase_offset,
            ),
            SoundEffectKind::Chorus(chorus) => modulated_delay(
                buffer,
                channels,
                sample_rate_hz,
                chorus.delay_frames,
                chorus.depth_frames.saturating_mul(chorus.voices as usize),
                chorus.rate_hz,
                0.0,
            ),
            SoundEffectKind::Delay(delay) => {
                delay_block(buffer, channels, delay.delay_frames, delay.feedback)
            }
            SoundEffectKind::PanStereo(pan) => pan_stereo(
                buffer,
                channels,
                pan.pan,
                pan.width,
                pan.left_gain,
                pan.right_gain,
                pan.invert_left_phase,
                pan.invert_right_phase,
            ),
            SoundEffectKind::Limiter(limiter) => limit(buffer, limiter.ceiling),
        }
        wet_mix(buffer, &dry, effect.wet);
    }
}

pub(crate) fn apply_track_controls(
    buffer: &mut [f32],
    channels: usize,
    controls: SoundTrackControls,
) {
    if controls.mute {
        buffer.fill(0.0);
        return;
    }
    if controls.delay_frames > 0 {
        delay_block(buffer, channels, controls.delay_frames, 0.0);
    }
    pan_stereo(
        buffer,
        channels,
        controls.pan,
        1.0,
        controls.left_gain * controls.gain,
        controls.right_gain * controls.gain,
        controls.invert_left_phase,
        controls.invert_right_phase,
    );
}

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

fn multiply(buffer: &mut [f32], gain: f32) {
    for sample in buffer {
        *sample *= gain;
    }
}

fn wet_mix(buffer: &mut [f32], dry: &[f32], wet: f32) {
    if wet >= 1.0 {
        return;
    }
    let dry_amount = 1.0 - wet;
    for (sample, dry_sample) in buffer.iter_mut().zip(dry.iter().copied()) {
        *sample = *sample * wet + dry_sample * dry_amount;
    }
}

fn filter_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    mode: SoundFilterMode,
    cutoff_hz: f32,
) {
    let rc = 1.0 / (cutoff_hz * std::f32::consts::TAU);
    let dt = 1.0 / sample_rate_hz.max(1) as f32;
    let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
    for channel in 0..channels {
        let mut low = 0.0;
        let mut previous_input = 0.0;
        for frame in 0..(buffer.len() / channels) {
            let index = frame * channels + channel;
            let input = buffer[index];
            low += alpha * (input - low);
            let high = alpha * (low + input - previous_input);
            buffer[index] = match mode {
                SoundFilterMode::LowPass | SoundFilterMode::LowShelf => low,
                SoundFilterMode::HighPass | SoundFilterMode::HighShelf => high,
                SoundFilterMode::BandPass => input - low - high,
                SoundFilterMode::Notch => low + high,
            };
            previous_input = input;
        }
    }
}

fn compressor_block(
    buffer: &mut [f32],
    channels: usize,
    threshold_db: f32,
    ratio: f32,
    makeup_gain_db: f32,
    sidechain: Option<&[f32]>,
) {
    let threshold = db_to_gain(threshold_db);
    let makeup = db_to_gain(makeup_gain_db);
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        let level = if let Some(sidechain) = sidechain {
            frame_level(sidechain, channels, frame)
        } else {
            frame_level(buffer, channels, frame)
        };
        let gain = if level > threshold && threshold > 0.0 {
            let over = level / threshold;
            let compressed = over.powf((1.0 / ratio.max(1.0)) - 1.0);
            compressed.clamp(0.0, 1.0)
        } else {
            1.0
        } * makeup;
        for channel in 0..channels {
            buffer[frame * channels + channel] *= gain;
        }
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

fn waveshape(buffer: &mut [f32], drive: f32) {
    let drive = drive.max(0.0) + 1.0;
    let normalizer = drive.tanh().max(0.0001);
    for sample in buffer {
        *sample = (*sample * drive).tanh() / normalizer;
    }
}

fn delay_block(buffer: &mut [f32], channels: usize, delay_frames: usize, feedback: f32) {
    if delay_frames == 0 {
        return;
    }
    let original = buffer.to_vec();
    for frame in 0..(buffer.len() / channels) {
        for channel in 0..channels {
            let index = frame * channels + channel;
            let delayed = frame
                .checked_sub(delay_frames)
                .and_then(|source_frame| original.get(source_frame * channels + channel))
                .copied()
                .unwrap_or_default();
            buffer[index] = delayed + original[index] * feedback.clamp(0.0, 0.99);
        }
    }
}

fn reverb_block(
    buffer: &mut [f32],
    channels: usize,
    pre_delay_frames: usize,
    tail_frames: usize,
    damping: f32,
) {
    let original = buffer.to_vec();
    let taps = [
        pre_delay_frames.max(1),
        tail_frames.max(2) / 2,
        tail_frames.max(3),
    ];
    for frame in 0..(buffer.len() / channels) {
        for channel in 0..channels {
            let mut wet = 0.0;
            for (tap_index, tap) in taps.iter().copied().enumerate() {
                if let Some(source_frame) = frame.checked_sub(tap) {
                    wet += original[source_frame * channels + channel]
                        * damping.clamp(0.0, 0.99).powi(tap_index as i32 + 1);
                }
            }
            buffer[frame * channels + channel] += wet;
        }
    }
}

fn convolve_block(buffer: &mut [f32], channels: usize, impulse_response: &[f32]) {
    if impulse_response.is_empty() {
        return;
    }
    let original = buffer.to_vec();
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        for channel in 0..channels {
            let mut sum = 0.0;
            for (tap, coefficient) in impulse_response.iter().copied().enumerate() {
                if let Some(source_frame) = frame.checked_sub(tap) {
                    sum += original[source_frame * channels + channel] * coefficient;
                }
            }
            buffer[frame * channels + channel] = sum;
        }
    }
}

fn modulated_delay(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    base_delay_frames: usize,
    depth_frames: usize,
    rate_hz: f32,
    feedback: f32,
) {
    let original = buffer.to_vec();
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        let phase = frame as f32 * rate_hz.max(0.0) / sample_rate_hz.max(1) as f32;
        let modulation = ((phase * std::f32::consts::TAU).sin() * 0.5 + 0.5) * depth_frames as f32;
        let delay = base_delay_frames + modulation.round() as usize;
        for channel in 0..channels {
            let delayed = frame
                .checked_sub(delay)
                .and_then(|source_frame| original.get(source_frame * channels + channel))
                .copied()
                .unwrap_or_default();
            let index = frame * channels + channel;
            buffer[index] = original[index] + delayed * (0.5 + feedback.clamp(0.0, 0.95) * 0.5);
        }
    }
}

fn phaser_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    rate_hz: f32,
    depth: f32,
    phase_offset: f32,
) {
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        let phase = (frame as f32 * rate_hz.max(0.0) / sample_rate_hz.max(1) as f32 + phase_offset)
            * std::f32::consts::TAU;
        let gain = 1.0 - depth.clamp(0.0, 1.0) * (phase.sin() * 0.5 + 0.5);
        for channel in 0..channels {
            buffer[frame * channels + channel] *= gain;
        }
    }
}

fn pan_stereo(
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

fn limit(buffer: &mut [f32], ceiling: f32) {
    let ceiling = ceiling.max(0.0);
    for sample in buffer {
        *sample = sample.clamp(-ceiling, ceiling);
    }
}
