use toml::Value;

pub(super) fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        _ => None,
    }
}

pub(super) fn value_as_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(value) => Some(*value),
        _ => None,
    }
}

pub(super) fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Float(value) => Some(*value),
        Value::Integer(value) => Some(*value as f64),
        _ => None,
    }
}

pub(super) fn value_as_float_array(value: &Value) -> Option<Vec<f32>> {
    let Value::Array(values) = value else {
        return None;
    };
    let components = values
        .iter()
        .filter_map(value_as_f64)
        .map(|value| value as f32)
        .collect::<Vec<_>>();
    if components.is_empty() {
        None
    } else {
        Some(components)
    }
}

pub(super) fn normalized_value_percent(value: f64, min: Option<f64>, max: Option<f64>) -> f32 {
    match (min, max) {
        (Some(min), Some(max)) if max > min => ((value - min) / (max - min)).clamp(0.0, 1.0) as f32,
        _ => value.clamp(0.0, 1.0) as f32,
    }
}

pub(super) fn value_as_color(value: &Value) -> Option<crate::ui::retained_host::primitives::Color> {
    let Value::String(value) = value else {
        return None;
    };
    parse_hex_color(value)
}

fn parse_hex_color(value: &str) -> Option<crate::ui::retained_host::primitives::Color> {
    let hex = value.strip_prefix('#')?;
    let hex = hex.as_bytes();
    match hex.len() {
        6 => Some(crate::ui::retained_host::primitives::Color::from_rgb_u8(
            decode_hex_byte(hex, 0)?,
            decode_hex_byte(hex, 2)?,
            decode_hex_byte(hex, 4)?,
        )),
        8 => Some(crate::ui::retained_host::primitives::Color::from_argb_u8(
            decode_hex_byte(hex, 6)?,
            decode_hex_byte(hex, 0)?,
            decode_hex_byte(hex, 2)?,
            decode_hex_byte(hex, 4)?,
        )),
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

pub(super) fn value_as_options(value: &Value) -> Option<Vec<String>> {
    let Value::Array(values) = value else {
        return None;
    };
    let options = values
        .iter()
        .filter_map(value_as_string)
        .collect::<Vec<_>>();
    if options.is_empty() {
        None
    } else {
        Some(options)
    }
}

#[cfg(test)]
#[path = "pane_value_conversion/direct_hex_color_tests.rs"]
mod direct_hex_color_tests;
