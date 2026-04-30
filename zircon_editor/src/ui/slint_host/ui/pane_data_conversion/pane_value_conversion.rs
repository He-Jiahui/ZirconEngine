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

pub(super) fn value_as_color(value: &Value) -> Option<slint::Color> {
    parse_hex_color(value_as_string(value)?.as_str())
}

fn parse_hex_color(value: &str) -> Option<slint::Color> {
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        6 => Some(slint::Color::from_rgb_u8(
            parse_hex_pair(&hex[0..2])?,
            parse_hex_pair(&hex[2..4])?,
            parse_hex_pair(&hex[4..6])?,
        )),
        8 => Some(slint::Color::from_argb_u8(
            parse_hex_pair(&hex[6..8])?,
            parse_hex_pair(&hex[0..2])?,
            parse_hex_pair(&hex[2..4])?,
            parse_hex_pair(&hex[4..6])?,
        )),
        _ => None,
    }
}

fn parse_hex_pair(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
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
