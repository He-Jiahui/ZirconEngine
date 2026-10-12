pub(super) fn variant_contains(component_variant: &str, expected: &str) -> bool {
    component_variant
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case(expected))
}

pub(super) fn value_as_i32(value: &toml::Value) -> Option<i32> {
    match value {
        toml::Value::Integer(value) => i32::try_from(*value).ok(),
        toml::Value::Float(value) => value.is_finite().then_some(value.round() as i32),
        toml::Value::String(value) => value.trim().parse::<i32>().ok(),
        _ => None,
    }
}
