use std::fmt::Write as _;

const CIRCULAR_PROGRESS_KEY_PREFIX: &str = "mui-circular-progress:";
const FIELD_SEPARATOR_COUNT: usize = 3;
const PERCENT_HEX_LEN: usize = 8;
const COLOR_HEX_LEN: usize = 8;

pub(super) fn circular_progress_image_key(
    size: u32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> String {
    let mut key = String::with_capacity(circular_progress_image_key_capacity(size));
    key.push_str(CIRCULAR_PROGRESS_KEY_PREFIX);
    write!(
        &mut key,
        "{size}:{:08x}:{:02x}{:02x}{:02x}{:02x}:{:02x}{:02x}{:02x}{:02x}",
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
    .expect("writing to a String cannot fail");
    key
}

fn circular_progress_image_key_capacity(size: u32) -> usize {
    CIRCULAR_PROGRESS_KEY_PREFIX.len()
        + decimal_digits(size)
        + FIELD_SEPARATOR_COUNT
        + PERCENT_HEX_LEN
        + COLOR_HEX_LEN * 2
}

fn decimal_digits(value: u32) -> usize {
    if value == 0 {
        1
    } else {
        value.ilog10() as usize + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_progress_cache_key_wire_format_is_stable() {
        let key = circular_progress_image_key(32, 0.5, [1, 2, 3, 4], [5, 6, 7, 8]);

        assert_eq!(key, "mui-circular-progress:32:3f000000:01020304:05060708");
        assert_eq!(key.len(), key.capacity());
    }

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
