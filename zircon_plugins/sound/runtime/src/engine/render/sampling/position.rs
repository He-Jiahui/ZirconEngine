pub(in crate::engine::render) fn next_source_frame_position(
    cursor_position: &mut f64,
    frame_count: usize,
    step: f64,
    looped: bool,
) -> Option<f64> {
    if frame_count == 0 {
        return None;
    }
    if *cursor_position >= frame_count as f64 {
        if looped {
            *cursor_position %= frame_count as f64;
        } else {
            return None;
        }
    }
    let frame_position = *cursor_position;
    *cursor_position += step;
    Some(frame_position)
}

pub(in crate::engine::render) fn next_clip_source_frame_position(
    cursor_position: &mut f64,
    frame_count: usize,
    range_start_frame: usize,
    range_end_frame: Option<usize>,
    step: f64,
    looped: bool,
) -> Option<f64> {
    if frame_count == 0 {
        return None;
    }
    let start = range_start_frame.min(frame_count);
    let end = range_end_frame
        .unwrap_or(frame_count)
        .min(frame_count)
        .max(start);
    if start >= end {
        return None;
    }
    if *cursor_position < start as f64 {
        *cursor_position = start as f64;
    }
    if *cursor_position >= end as f64 {
        if looped {
            *cursor_position = start as f64;
        } else {
            return None;
        }
    }
    let frame_position = *cursor_position;
    *cursor_position += step;
    Some(frame_position)
}
