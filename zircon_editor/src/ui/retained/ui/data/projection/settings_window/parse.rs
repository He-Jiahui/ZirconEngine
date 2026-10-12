use toml::Value;

pub(super) fn string_value(values: &toml::map::Map<String, Value>, key: &str) -> String {
    values
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
        .to_owned()
}

pub(super) fn bool_value(values: &toml::map::Map<String, Value>, key: &str) -> bool {
    values.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub(super) fn rgba_value(values: &toml::map::Map<String, Value>, key: &str) -> Option<[u8; 4]> {
    let channels = values.get(key)?.as_array()?;
    let [red, green, blue, alpha] = channels.as_slice() else {
        return None;
    };
    Some([
        rgba_channel(red)?,
        rgba_channel(green)?,
        rgba_channel(blue)?,
        rgba_channel(alpha)?,
    ])
}

fn rgba_channel(value: &Value) -> Option<u8> {
    value
        .as_integer()
        .and_then(|value| u8::try_from(value).ok())
}

pub(super) fn string_array(
    values: &toml::map::Map<String, Value>,
    key: &str,
) -> Vec<crate::ui::retained_host::primitives::SharedString> {
    values
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(Into::into)
        .collect()
}

pub(super) fn table_array<'a>(
    attributes: &'a std::collections::BTreeMap<String, Value>,
    key: &str,
) -> impl Iterator<Item = &'a toml::map::Map<String, Value>> {
    attributes
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_table)
}
