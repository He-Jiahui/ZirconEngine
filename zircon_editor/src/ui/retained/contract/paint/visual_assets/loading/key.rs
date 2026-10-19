use std::fmt::Write as _;

use super::super::RasterTargetSize;

const INTRINSIC_SIZE: &str = "intrinsic";
const NO_TINT: &str = "none";
const TINT_MARKER: &str = ":tint:";
const TINT_HEX_LEN: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VisualAssetVariant {
    pub(super) target: Option<RasterTargetSize>,
    pub(super) tint: Option<[u8; 4]>,
}

impl VisualAssetVariant {
    pub(super) fn from_cache_key(key: &str, base_key: &str) -> Option<Self> {
        let suffix = key.strip_prefix(base_key)?.strip_prefix(':')?;
        let (size, tint) = suffix.split_once(TINT_MARKER)?;
        let target = if size == INTRINSIC_SIZE {
            None
        } else {
            let (width, height) = size.split_once('x')?;
            Some(RasterTargetSize {
                width: width.parse().ok()?,
                height: height.parse().ok()?,
            })
        };
        let tint = if tint == NO_TINT {
            None
        } else {
            if tint.len() != TINT_HEX_LEN || !tint.is_ascii() {
                return None;
            }
            let mut rgba = [0; 4];
            for (index, value) in rgba.iter_mut().enumerate() {
                *value = u8::from_str_radix(&tint[index * 2..index * 2 + 2], 16).ok()?;
            }
            Some(rgba)
        };
        Some(Self { target, tint })
    }
}

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
