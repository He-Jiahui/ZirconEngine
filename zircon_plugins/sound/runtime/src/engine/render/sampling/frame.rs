pub(super) fn source_frame_sample(
    samples: &[f32],
    source_channels: usize,
    frame_index: usize,
    output_channel: usize,
    output_channel_count: usize,
) -> f32 {
    let source_frame_offset = frame_index.saturating_mul(source_channels);
    let source_frame_end = source_frame_offset.saturating_add(source_channels);
    let Some(source_frame) = samples.get(source_frame_offset..source_frame_end) else {
        return 0.0;
    };
    sample_for_output_channel(source_frame, output_channel, output_channel_count)
}

fn sample_for_output_channel(
    clip_frame: &[f32],
    output_channel: usize,
    output_channel_count: usize,
) -> f32 {
    if clip_frame.len() == 1 {
        return clip_frame[0];
    }
    if output_channel_count == 1 {
        return clip_frame.iter().copied().sum::<f32>() / clip_frame.len() as f32;
    }

    clip_frame
        .get(output_channel)
        .copied()
        .unwrap_or_else(|| *clip_frame.last().unwrap_or(&0.0))
}
