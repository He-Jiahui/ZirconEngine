use zircon_runtime::core::framework::script::ScriptHostValue;
use zr_vm_rust_binding as zrvm;

use super::errors::zr_error;

pub(super) fn read_host_arguments_for_function(
    context: &zrvm::NativeCallContext,
    function_label: &str,
) -> Result<Vec<ScriptHostValue>, zrvm::Error> {
    let count = context.argument_count().map_err(|error| {
        zr_error(format!(
            "failed to read argument count for {function_label}: {error}"
        ))
    })?;
    let mut arguments = Vec::with_capacity(count);
    for index in 0..count {
        let value = context.argument(index).map_err(|error| {
            zr_error(format!(
                "failed to read argument {index} for {function_label}: {error}"
            ))
        })?;
        arguments.push(from_zr_value_for_function(&value, function_label, index)?);
    }
    Ok(arguments)
}

pub(super) fn from_zr_value_for_function(
    value: &zrvm::Value,
    function_label: &str,
    index: usize,
) -> Result<ScriptHostValue, zrvm::Error> {
    let value_label = format!("{function_label} argument {index}");
    from_zr_value(value, &value_label)
}

pub(super) fn from_zr_return_value_for_export(
    value: &zrvm::Value,
    export_label: &str,
) -> Result<ScriptHostValue, zrvm::Error> {
    from_zr_value(value, &format!("export {export_label}"))
}

fn from_zr_value(value: &zrvm::Value, value_label: &str) -> Result<ScriptHostValue, zrvm::Error> {
    match value.kind() {
        zrvm::ValueKind::Null => Ok(ScriptHostValue::Null),
        zrvm::ValueKind::Bool => Ok(ScriptHostValue::Bool(value.as_bool()?)),
        zrvm::ValueKind::Int => Ok(ScriptHostValue::Int(value.as_int()?)),
        zrvm::ValueKind::Float => Ok(ScriptHostValue::Float(value.as_float()?)),
        zrvm::ValueKind::String => Ok(ScriptHostValue::String(value.as_string()?)),
        zrvm::ValueKind::Array => from_zr_byte_array(value, value_label),
        other => Err(zr_error(format!(
            "unsupported zr_vm value kind {other:?} at {value_label}"
        ))),
    }
}

fn from_zr_byte_array(
    value: &zrvm::Value,
    value_label: &str,
) -> Result<ScriptHostValue, zrvm::Error> {
    let length = value.array_len().map_err(|error| {
        zr_error(format!(
            "failed to read byte array length at {value_label}: {error}"
        ))
    })?;
    let mut bytes = Vec::with_capacity(length);
    for index in 0..length {
        let item = value.array_get(index).map_err(|error| {
            zr_error(format!(
                "failed to read byte array element {index} at {value_label}: {error}"
            ))
        })?;
        let integer = item.as_int().map_err(|error| {
            zr_error(format!(
                "expected byte integer at {value_label} element {index}: {error}"
            ))
        })?;
        let byte = u8::try_from(integer).map_err(|_| {
            zr_error(format!(
                "byte array element {index} at {value_label} is outside 0..=255: {integer}"
            ))
        })?;
        bytes.push(byte);
    }
    Ok(ScriptHostValue::Bytes(bytes))
}

pub(super) fn to_zr_value_for_function(
    value: ScriptHostValue,
    function_label: &str,
) -> Result<zrvm::Value, zrvm::Error> {
    to_zr_value(value).map_err(|error| {
        zr_error(format!(
            "failed to lower host return value for {function_label}: {error}"
        ))
    })
}

pub(super) fn to_zr_value(value: ScriptHostValue) -> Result<zrvm::Value, zrvm::Error> {
    match value {
        ScriptHostValue::Null => zrvm::Value::new_null(),
        ScriptHostValue::Bool(value) => zrvm::Value::new_bool(value),
        ScriptHostValue::Int(value) => zrvm::Value::new_int(value),
        ScriptHostValue::Float(value) => zrvm::Value::new_float(value),
        ScriptHostValue::String(value) => zrvm::Value::new_string(&value),
        ScriptHostValue::Bytes(value) => {
            let mut bytes = zrvm::Value::new_array()?;
            for byte in value {
                bytes.array_push(&zrvm::Value::new_int(i64::from(byte))?)?;
            }
            Ok(bytes)
        }
        ScriptHostValue::HostHandle(value) => zrvm::Value::new_int(value as i64),
    }
}
