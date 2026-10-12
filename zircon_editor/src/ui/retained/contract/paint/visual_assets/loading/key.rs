use std::fmt::Write as _;

use super::super::RasterTargetSize;

const INTRINSIC_SIZE: &str = "intrinsic";
const NO_TINT: &str = "none";
const TINT_MARKER: &str = ":tint:";
const TINT_HEX_LEN: usize = 8;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_pixels_cache_key(
    base_key: &str,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> String {
    let mut key = String::with_capacity(image_pixels_cache_key_capacity(base_key, target, tint));
    key.push_str(base_key);
    key.push(':');
    match target {
        Some(target) => {
            write!(&mut key, "{}x{}", target.width, target.height)
                .expect("writing to a String cannot fail");
        }
        None => key.push_str(INTRINSIC_SIZE),
    }
    key.push_str(TINT_MARKER);
    match tint {
        Some(tint) => {
            write!(
                &mut key,
                "{:02x}{:02x}{:02x}{:02x}",
                tint[0], tint[1], tint[2], tint[3]
            )
            .expect("writing to a String cannot fail");
        }
        None => key.push_str(NO_TINT),
    }
    key
}

fn image_pixels_cache_key_capacity(
    base_key: &str,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> usize {
    let size_len = target.map_or(INTRINSIC_SIZE.len(), |target| {
        decimal_digits(target.width) + 1 + decimal_digits(target.height)
    });
    let tint_len = tint.map_or(NO_TINT.len(), |_| TINT_HEX_LEN);
    base_key.len() + 1 + size_len + TINT_MARKER.len() + tint_len
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
