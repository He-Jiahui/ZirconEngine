pub(super) fn assert_complete_frames(channel_count: u16, samples: &[f32]) {
    assert_ne!(channel_count, 0, "test clip channel count must be non-zero");
    assert_eq!(
        samples.len() % channel_count as usize,
        0,
        "test clip samples must contain complete frames"
    );
}
