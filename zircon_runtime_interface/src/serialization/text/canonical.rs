use serde_json::Value;

/// Recursively orders every object key before the text formatter runs.
pub(in crate::serialization) fn canonicalize_value(mut value: Value) -> Value {
    canonicalize_value_in_place(&mut value);
    value
}

fn canonicalize_value_in_place(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                canonicalize_nested_value(value);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                canonicalize_nested_value(value);
            }
            values.sort_keys();
        }
        _ => {}
    }
}

#[inline]
fn canonicalize_nested_value(value: &mut Value) {
    if matches!(value, Value::Array(_) | Value::Object(_)) {
        canonicalize_value_in_place(value);
    }
}

#[cfg(test)]
#[path = "tests/canonical_performance_tests.rs"]
mod performance_tests;
