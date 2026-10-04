use std::fmt::Write as _;

const CIRCULAR_PROGRESS_KEY_PREFIX: &str = "mui-circular-progress:";
const FIELD_SEPARATOR_COUNT: usize = 4;
const PERCENT_HEX_LEN: usize = 8;
const COLOR_HEX_LEN: usize = 8;

pub(super) fn circular_progress_image_key(
    size: u32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> String {
    circular_progress_image_key_for_target(size, size as f32, percent, track, fill)
}

pub(super) fn circular_progress_image_key_for_target(
    source_size: u32,
    target_size: f32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> String {
    let mut key = String::with_capacity(circular_progress_image_key_capacity(source_size));
    key.push_str(CIRCULAR_PROGRESS_KEY_PREFIX);
    write!(
        &mut key,
        "{source_size}:{:08x}:{:08x}:{:02x}{:02x}{:02x}{:02x}:{:02x}{:02x}{:02x}{:02x}",
        target_size.to_bits(),
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
        + PERCENT_HEX_LEN * 2
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
#[path = "tests/key.rs"]
mod tests;
