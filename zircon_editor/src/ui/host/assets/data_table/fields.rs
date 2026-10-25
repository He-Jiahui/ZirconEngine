use std::collections::HashSet;

use serde_json::value::RawValue;
use zircon_runtime_interface::reflect::{
    ReflectFieldInfo, ReflectTypeInfo, ReflectTypeKind, ReflectedValue,
};

use super::DataTableSourceError;

pub(super) fn columns<'a>(
    row_type: &'a ReflectTypeInfo,
    key_field: &str,
) -> Result<Vec<&'a ReflectFieldInfo>, DataTableSourceError> {
    if row_type.kind != ReflectTypeKind::Struct || key_field.is_empty() {
        return Err(DataTableSourceError::InvalidRowType(
            "a struct row type and explicit nonempty key field are required".into(),
        ));
    }
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for field in &row_type.fields {
        if field.name.is_empty()
            || field.name == key_field
            || !ids.insert(field.id)
            || !names.insert(field.name.as_str())
        {
            return Err(DataTableSourceError::InvalidRowType(format!(
                "duplicate/empty field identity or key collision: {}",
                field.name
            )));
        }
        if field.serializable && !supported(&field.value_type_path) {
            return Err(DataTableSourceError::UnsupportedColumn {
                field: field.name.clone(),
                value_type_path: field.value_type_path.clone(),
            });
        }
    }
    Ok(row_type
        .fields
        .iter()
        .filter(|field| field.serializable)
        .collect())
}

fn supported(path: &str) -> bool {
    matches!(
        path,
        "bool"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "f32"
            | "alloc::string::String"
    )
}

pub(super) fn value(raw: &RawValue, field: &ReflectFieldInfo) -> Result<ReflectedValue, String> {
    let text = raw.get();
    macro_rules! parse {
        ($ty:ty, $variant:ident) => {
            serde_json::from_str::<$ty>(text)
                .map(|value| ReflectedValue::$variant(value.into()))
                .map_err(|error| error.to_string())
        };
    }
    match field.value_type_path.as_str() {
        "bool" => parse!(bool, Bool),
        "i8" => parse!(i8, Integer),
        "i16" => parse!(i16, Integer),
        "i32" => parse!(i32, Integer),
        "i64" => parse!(i64, Integer),
        "u8" => parse!(u8, Unsigned),
        "u16" => parse!(u16, Unsigned),
        "u32" => parse!(u32, Unsigned),
        "u64" => parse!(u64, Unsigned),
        "alloc::string::String" => parse!(String, String),
        "f32" => exact_scalar(text).map(ReflectedValue::Scalar),
        _ => Err("unsupported column type".into()),
    }
}

fn exact_scalar(text: &str) -> Result<f32, String> {
    // Validate the original token, before any f64 conversion could erase decimal digits.
    let scalar = serde_json::from_str::<f32>(text).map_err(|error| error.to_string())?;
    if !scalar.is_finite() {
        return Err("number exceeds finite f32 range".into());
    }
    let decimal = normalized_decimal(text).ok_or("unsupported numeric spelling")?;
    if Some(&decimal) != normalized_decimal(&scalar.to_string()).as_ref()
        && Some(&decimal) != normalized_decimal(&format!("{scalar:.149}")).as_ref()
    {
        return Err("number loses precision in f32".into());
    }
    Ok(scalar)
}

fn normalized_decimal(text: &str) -> Option<(bool, String, i32)> {
    let (negative, text) = text
        .strip_prefix('-')
        .map_or((false, text), |text| (true, text));
    let (mantissa, exponent) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i32>().ok()?),
        None => (text, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let coefficient = format!("{whole}{fraction}");
    if coefficient.is_empty() || !coefficient.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let coefficient = coefficient.trim_start_matches('0');
    if coefficient.is_empty() {
        return Some((false, "0".into(), 0));
    }
    let trimmed = coefficient.trim_end_matches('0');
    let exponent = exponent
        .checked_sub(i32::try_from(fraction.len()).ok()?)?
        .checked_add(i32::try_from(coefficient.len() - trimmed.len()).ok()?)?;
    Some((negative, trimmed.to_string(), exponent))
}
