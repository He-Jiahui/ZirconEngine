use super::frame::source_frame_sample;

pub(in crate::engine::render) fn interpolated_source_sample(
    samples: &[f32],
    source_channels: usize,
    frame_count: usize,
    range_start_frame: usize,
    range_end_frame: Option<usize>,
    frame_position: f64,
    output_channel: usize,
    output_channel_count: usize,
    looped: bool,
) -> f32 {
    if source_channels == 0 || frame_count == 0 {
        return 0.0;
    }

    let base_position = frame_position.floor().max(0.0);
    let base_frame = (base_position as usize).min(frame_count - 1);
    let range_start = range_start_frame.min(frame_count);
    let range_end = range_end_frame
        .unwrap_or(frame_count)
        .min(frame_count)
        .max(range_start);
    let next_frame = if base_frame + 1 < range_end {
        base_frame + 1
    } else if looped {
        range_start
    } else {
        base_frame
    };
    let blend = (frame_position - base_position).clamp(0.0, 1.0) as f32;
    let start = source_frame_sample(
        samples,
        source_channels,
        base_frame,
        output_channel,
        output_channel_count,
    );
    let end = source_frame_sample(
        samples,
        source_channels,
        next_frame,
        output_channel,
        output_channel_count,
    );
    start + (end - start) * blend
}
