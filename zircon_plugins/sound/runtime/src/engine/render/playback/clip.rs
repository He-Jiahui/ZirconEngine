use zircon_runtime::asset::SoundAsset;

use crate::SoundConfig;

use super::super::super::state::ActivePlayback;
use super::super::sampling::{
    interpolated_source_sample, next_clip_source_frame_position, resample_step,
};
use super::pan::playback_channel_gain;

pub(in crate::engine::render) fn mix_clip_playback(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    clip: &SoundAsset,
    playback: &mut ActivePlayback,
    config: &SoundConfig,
) -> bool {
    if playback.paused {
        return false;
    }
    let clip_channels = clip.channel_count as usize;
    let frame_count = clip.frame_count();
    if frame_count == 0 || clip_channels == 0 {
        return true;
    }
    let step = resample_step(clip.sample_rate_hz, config.sample_rate_hz) * playback.speed as f64;

    for frame_index in 0..frames {
        let Some(source_frame_position) = next_clip_source_frame_position(
            &mut playback.cursor_position,
            frame_count,
            playback.range_start_frame,
            playback.range_end_frame,
            step,
            playback.looped,
        ) else {
            playback.cursor_frame = playback.range_end_frame.unwrap_or(frame_count);
            return true;
        };

        let output_offset = frame_index * output_channels;
        for channel in 0..output_channels {
            let mut sample = interpolated_source_sample(
                &clip.samples,
                clip_channels,
                frame_count,
                playback.range_start_frame,
                playback.range_end_frame,
                source_frame_position,
                channel,
                output_channels,
                playback.looped,
            );
            sample *= playback.gain;
            if output_channels > 1 {
                sample *= playback_channel_gain(playback.pan, channel);
            }
            if !playback.muted {
                destination[output_offset + channel] += sample;
            }
        }
        playback.cursor_frame = playback.cursor_position.floor() as usize;
    }

    false
}
