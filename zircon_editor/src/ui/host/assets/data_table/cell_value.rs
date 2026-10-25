use serde_json::value::RawValue;
use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectedValue};

use super::fields;

pub(super) fn equal(left: &ReflectedValue, right: &ReflectedValue) -> bool {
    match (left, right) {
        // Preserve a deliberate signed-zero edit instead of collapsing its original source.
        (ReflectedValue::Scalar(left), ReflectedValue::Scalar(right)) => {
            left.to_bits() == right.to_bits()
        }
        _ => left == right,
    }
}

pub(super) fn encode(field: &ReflectFieldInfo, value: &ReflectedValue) -> Result<String, String> {
    use ReflectedValue::*;
    macro_rules! integer {
        ($ty:ty, $value:expr) => {
            serde_json::to_string(
                &<$ty>::try_from(*$value).map_err(|_| "integer exceeds declared field width")?,
            )
            .map_err(|error| error.to_string())
        };
    }
    let text = match (field.value_type_path.as_str(), value) {
        ("i8", Integer(value)) => integer!(i8, value),
        ("i16", Integer(value)) => integer!(i16, value),
        ("i32", Integer(value)) => integer!(i32, value),
        ("i64", Integer(value)) => integer!(i64, value),
        ("u8", Unsigned(value)) => integer!(u8, value),
        ("u16", Unsigned(value)) => integer!(u16, value),
        ("u32", Unsigned(value)) => integer!(u32, value),
        ("u64", Unsigned(value)) => integer!(u64, value),
        ("bool", Bool(value)) => serde_json::to_string(value).map_err(|error| error.to_string()),
        ("alloc::string::String", String(value)) => {
            serde_json::to_string(value).map_err(|error| error.to_string())
        }
        ("f32", Scalar(value)) if value.is_finite() => {
            serde_json::to_string(value).map_err(|error| error.to_string())
        }
        _ => Err(format!(
            "expected {}, got {} (finite values required)",
            field.value_type_path,
            value.type_name()
        )),
    }?;
    let raw: &RawValue = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    if !equal(&fields::value(raw, field)?, value) {
        return Err("encoded cell does not round trip to its original typed value".into());
    }
    bounds(field, value)?;
    Ok(text)
}

fn bounds(field: &ReflectFieldInfo, value: &ReflectedValue) -> Result<(), String> {
    let Some(range) = &field.numeric_range else {
        return Ok(());
    };
    let within = match value {
        ReflectedValue::Scalar(value) => {
            !range.min().is_some_and(|min| *value < min)
                && !range.max().is_some_and(|max| *value > max)
        }
        ReflectedValue::Integer(value) => integer_bounds(i128::from(*value), range),
        ReflectedValue::Unsigned(value) => integer_bounds(i128::from(*value), range),
        _ => return Err("numeric range requires a numeric column".into()),
    };
    if !within {
        return Err("value is outside the schema numeric range".into());
    }
    // Step/precision are editor hints, not permission to round stored authoring values.
    Ok(())
}

fn integer_bounds(
    value: i128,
    range: &zircon_runtime_interface::reflect::ReflectNumericRange,
) -> bool {
    // Every supported i64/u64 fits i128. Float bounds outside i128 saturate beyond that entire
    // domain; ceil/floor make fractional limits exact without converting the integer to f32/f64.
    !range.min().is_some_and(|min| value < (min.ceil() as i128))
        && !range.max().is_some_and(|max| value > (max.floor() as i128))
}
