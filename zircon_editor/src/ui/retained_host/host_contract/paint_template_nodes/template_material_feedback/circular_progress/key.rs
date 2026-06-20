pub(super) fn circular_progress_image_key(
    size: u32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> String {
    format!(
        "mui-circular-progress:{size}:{percent:.3}:{}:{}",
        track[0], fill[0]
    )
}
