pub(super) fn circular_progress_image_key(
    size: u32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> String {
    format!(
        "mui-circular-progress:{size}:{:08x}:{:02x}{:02x}{:02x}{:02x}:{:02x}{:02x}{:02x}{:02x}",
        percent.to_bits(),
        track[0],
        track[1],
        track[2],
        track[3],
        fill[0],
        fill[1],
        fill[2],
        fill[3],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_progress_key_includes_complete_color_and_progress_identity() {
        let baseline = circular_progress_image_key(32, 0.58, [10, 20, 30, 40], [50, 60, 70, 80]);

        assert_ne!(
            baseline,
            circular_progress_image_key(32, 0.58, [10, 21, 30, 40], [50, 60, 70, 80])
        );
        assert_ne!(
            baseline,
            circular_progress_image_key(32, 0.58, [10, 20, 30, 40], [50, 61, 70, 80])
        );
        assert_ne!(
            baseline,
            circular_progress_image_key(32, 0.59, [10, 20, 30, 40], [50, 60, 70, 80])
        );
    }

    #[test]
    fn circular_progress_entry_keys_the_resolved_raster_percent() {
        let production = include_str!("entry.rs");

        assert!(production.contains("circular_progress_image_key(size, progress, track, fill)"));
    }
}
