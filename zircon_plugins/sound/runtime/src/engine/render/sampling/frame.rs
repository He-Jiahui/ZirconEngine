use zircon_runtime::core::framework::audio::AudioChannelLayout;

use super::super::channel_layout::source_frame_sample_for_output;

pub(super) fn source_frame_sample(
    samples: &[f32],
    source_channels: usize,
    source_layout: &AudioChannelLayout,
    frame_index: usize,
    output_channel: usize,
    output_layout: &AudioChannelLayout,
) -> f32 {
    let source_frame_offset = frame_index.saturating_mul(source_channels);
    let source_frame_end = source_frame_offset.saturating_add(source_channels);
    let Some(source_frame) = samples.get(source_frame_offset..source_frame_end) else {
        return 0.0;
    };
    source_frame_sample_for_output(source_frame, source_layout, output_layout, output_channel)
}
