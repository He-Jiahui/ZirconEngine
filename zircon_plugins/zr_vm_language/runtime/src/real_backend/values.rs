use zircon_runtime::core::framework::script::{
    ScriptHostArgumentSource, ScriptHostByteSource, ScriptHostByteView, ScriptHostError,
    ScriptHostValue, ScriptHostValueRef,
};
use zr_vm_rust_binding as zrvm;

use super::errors::zr_error;

pub(super) struct ZrVmScriptHostArgumentSource<'context, 'native> {
    context: &'context zrvm::NativeCallContext<'native>,
    function_label: &'context str,
    argument_count: usize,
}

impl<'context, 'native> ZrVmScriptHostArgumentSource<'context, 'native> {
    pub(super) fn new(
        context: &'context zrvm::NativeCallContext<'native>,
        function_label: &'context str,
    ) -> Result<Self, zrvm::Error> {
        let argument_count = context.argument_count().map_err(|error| {
            zr_error(format!(
                "failed to read argument count for {function_label}: {error}"
            ))
        })?;
        Ok(Self {
            context,
            function_label,
            argument_count,
        })
    }
}

impl ScriptHostArgumentSource for ZrVmScriptHostArgumentSource<'_, '_> {
    fn len(&self) -> usize {
        self.argument_count
    }

    fn visit_argument(
        &self,
        index: usize,
        visitor: &mut dyn for<'argument> FnMut(
            ScriptHostValueRef<'argument>,
        ) -> Result<(), ScriptHostError>,
    ) -> Result<(), ScriptHostError> {
        if index >= self.argument_count {
            return Err(ScriptHostError::new(format!(
                "{} argument {index} was not provided",
                self.function_label
            )));
        }

        self.context
            .with_argument(index, |argument| match argument.kind()? {
                zrvm::NativeArgumentKind::Null => {
                    visitor(ScriptHostValueRef::Null).map_err(|error| zr_error(error.message))
                }
                zrvm::NativeArgumentKind::Bool => {
                    visitor(ScriptHostValueRef::Bool(argument.read_bool()?))
                        .map_err(|error| zr_error(error.message))
                }
                zrvm::NativeArgumentKind::Int => {
                    visitor(ScriptHostValueRef::Int(argument.read_int()?))
                        .map_err(|error| zr_error(error.message))
                }
                zrvm::NativeArgumentKind::Float => {
                    visitor(ScriptHostValueRef::Float(argument.read_float()?))
                        .map_err(|error| zr_error(error.message))
                }
                zrvm::NativeArgumentKind::String => argument.with_str(|value| {
                    visitor(ScriptHostValueRef::String(value))
                        .map_err(|error| zr_error(error.message))
                }),
                zrvm::NativeArgumentKind::Array => {
                    let bytes = ZrVmByteSource {
                        argument: &argument,
                    };
                    visitor(ScriptHostValueRef::Bytes(ScriptHostByteView::Source(
                        &bytes,
                    )))
                    .map_err(|error| zr_error(error.message))
                }
                kind => {
                    return Err(zr_error(format!(
                        "unsupported zr_vm value kind {kind:?} at {} argument {index}",
                        self.function_label
                    )));
                }
            })
            .map_err(|error| {
                ScriptHostError::new(format!(
                    "failed to read argument {index} for {}: {error}",
                    self.function_label
                ))
            })
    }
}

struct ZrVmByteSource<'source, 'argument> {
    argument: &'source zrvm::NativeArgumentView<'argument>,
}

impl ScriptHostByteSource for ZrVmByteSource<'_, '_> {
    fn len(&self) -> Result<usize, ScriptHostError> {
        self.argument.byte_len().map_err(|error| {
            ScriptHostError::new(format!("failed to read byte-array length: {error}"))
        })
    }

    fn byte_at(&self, index: usize) -> Result<u8, ScriptHostError> {
        self.argument.byte_at(index).map_err(|error| {
            ScriptHostError::new(format!(
                "failed to read byte-array element {index}: {error}"
            ))
        })
    }
}

pub(super) fn from_zr_return_value_for_export(
    value: &zrvm::Value,
    module_name: &str,
    export_name: &str,
) -> Result<ScriptHostValue, zrvm::Error> {
    from_zr_value(value, module_name, export_name)
}

fn from_zr_value(
    value: &zrvm::Value,
    module_name: &str,
    export_name: &str,
) -> Result<ScriptHostValue, zrvm::Error> {
    match value.kind() {
        zrvm::ValueKind::Null => Ok(ScriptHostValue::Null),
        zrvm::ValueKind::Bool => Ok(ScriptHostValue::Bool(value.as_bool()?)),
        zrvm::ValueKind::Int => Ok(ScriptHostValue::Int(value.as_int()?)),
        zrvm::ValueKind::Float => Ok(ScriptHostValue::Float(value.as_float()?)),
        zrvm::ValueKind::String => Ok(ScriptHostValue::String(value.as_string()?)),
        zrvm::ValueKind::Array => from_zr_byte_array(value, module_name, export_name),
        other => Err(zr_error(format!(
            "unsupported zr_vm value kind {other:?} at export {module_name}.{export_name}"
        ))),
    }
}

fn from_zr_byte_array(
    value: &zrvm::Value,
    module_name: &str,
    export_name: &str,
) -> Result<ScriptHostValue, zrvm::Error> {
    let length = value.array_len().map_err(|error| {
        zr_error(format!(
            "failed to read byte array length at export {module_name}.{export_name}: {error}"
        ))
    })?;
    let mut bytes = Vec::with_capacity(length);
    for index in 0..length {
        let item = value.array_get(index).map_err(|error| {
            zr_error(format!(
                "failed to read byte array element {index} at export {module_name}.{export_name}: {error}"
            ))
        })?;
        let integer = item.as_int().map_err(|error| {
            zr_error(format!(
                "expected byte integer at export {module_name}.{export_name} element {index}: {error}"
            ))
        })?;
        let byte = u8::try_from(integer).map_err(|_| {
            zr_error(format!(
                "byte array element {index} at export {module_name}.{export_name} is outside 0..=255: {integer}"
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
    to_zr_value(&value).map_err(|error| {
        zr_error(format!(
            "failed to lower host return value for {function_label}: {error}"
        ))
    })
}

pub(super) fn to_zr_value(value: &ScriptHostValue) -> Result<zrvm::Value, zrvm::Error> {
    match value {
        ScriptHostValue::Null => zrvm::Value::new_null(),
        ScriptHostValue::Bool(value) => zrvm::Value::new_bool(*value),
        ScriptHostValue::Int(value) => zrvm::Value::new_int(*value),
        ScriptHostValue::Float(value) => zrvm::Value::new_float(*value),
        ScriptHostValue::String(value) => zrvm::Value::new_string(value),
        ScriptHostValue::Bytes(value) => {
            let mut bytes = zrvm::Value::new_array()?;
            for byte in value {
                bytes.array_push(&zrvm::Value::new_int(i64::from(*byte))?)?;
            }
            Ok(bytes)
        }
        ScriptHostValue::HostHandle(value) => zrvm::Value::new_int(*value as i64),
    }
}
