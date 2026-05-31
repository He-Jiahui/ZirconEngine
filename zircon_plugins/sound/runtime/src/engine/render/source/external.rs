use zircon_runtime::core::framework::sound::SoundExternalSourceBlock;

use super::super::sampling::{
    interpolated_source_sample, next_source_frame_position, resample_step,
};

pub(super) fn mix_external_source_block(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    block: &SoundExternalSourceBlock,
    gain: f32,
    looped: bool,
    output_sample_rate_hz: u32,
    cursor_frame: &mut usize,
    cursor_position: &mut f64,
) -> bool {
    let source_channels = block.channel_count as usize;
    if source_channels == 0 {
        return !looped;
    }
    let frame_count = block.samples.len() / source_channels;
    if frame_count == 0 {
        return !looped;
    }
    let step = resample_step(block.sample_rate_hz, output_sample_rate_hz);

    for frame_index in 0..frames {
        let Some(source_frame_position) =
            next_source_frame_position(cursor_position, frame_count, step, looped)
        else {
            *cursor_frame = frame_count;
            return true;
        };

        let output_offset = frame_index * output_channels;
        for channel in 0..output_channels {
            destination[output_offset + channel] += interpolated_source_sample(
                &block.samples,
                source_channels,
                frame_count,
                0,
                None,
                source_frame_position,
                channel,
                output_channels,
                looped,
            ) * gain;
        }
        *cursor_frame = cursor_position.floor() as usize;
    }
    false
}
