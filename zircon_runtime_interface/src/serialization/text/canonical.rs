use serde_json::{Map, Value};

/// Recursively orders every object key before the text formatter runs.
pub(in crate::serialization) fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_value).collect()),
        Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            Value::Object(Map::from_iter(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonicalize_value(value))),
            ))
        }
        value => value,
    }
}
