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
    match hex.len() {
        6 => {
            let r = parse_hex_byte(&hex[0..2])?;
            let g = parse_hex_byte(&hex[2..4])?;
            let b = parse_hex_byte(&hex[4..6])?;
            Some([
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                opacity,
            ])
        }
        8 => {
            let r = parse_hex_byte(&hex[0..2])?;
            let g = parse_hex_byte(&hex[2..4])?;
            let b = parse_hex_byte(&hex[4..6])?;
            let a = parse_hex_byte(&hex[6..8])?;
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

fn parse_hex_byte(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}
