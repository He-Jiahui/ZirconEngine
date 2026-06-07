pub(super) fn positional_source_sample(source_frame: &[f32], output_channel: usize) -> f32 {
    source_frame
        .get(output_channel)
        .copied()
        .unwrap_or_default()
}
