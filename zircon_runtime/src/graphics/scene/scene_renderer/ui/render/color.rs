pub(super) fn parse_color(
    value: Option<&str>,
    fallback: [f32; 4],
    opacity: f32,
) -> Option<[f32; 4]> {
    parse_hex_color(value.unwrap_or(""), opacity).or_else(|| {
        (opacity > 0.0).then_some([fallback[0], fallback[1], fallback[2], fallback[3] * opacity])
    })
}

pub(super) fn parse_hex_color(value: &str, opacity: f32) -> Option<[f32; 4]> {
    let hex = value.strip_prefix('#')?;
    let hex = hex.as_bytes();
    match hex.len() {
        6 => {
            let r = decode_hex_byte(hex, 0)?;
            let g = decode_hex_byte(hex, 2)?;
            let b = decode_hex_byte(hex, 4)?;
            Some([
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                opacity,
            ])
        }
        8 => {
            let r = decode_hex_byte(hex, 0)?;
            let g = decode_hex_byte(hex, 2)?;
            let b = decode_hex_byte(hex, 4)?;
            let a = decode_hex_byte(hex, 6)?;
            Some([
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                (a as f32 / 255.0) * opacity,
            ])
        }
        _ => None,
    }
}

fn decode_hex_byte(encoded: &[u8], offset: usize) -> Option<u8> {
    let high = decode_hex_digit(*encoded.get(offset)?)?;
    let low = decode_hex_digit(*encoded.get(offset + 1)?)?;
    Some((high << 4) | low)
}

fn decode_hex_digit(encoded: u8) -> Option<u8> {
    match encoded {
        b'0'..=b'9' => Some(encoded - b'0'),
        b'a'..=b'f' => Some(encoded - b'a' + 10),
        b'A'..=b'F' => Some(encoded - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "color/direct_hex_color_tests.rs"]
mod direct_hex_color_tests;
