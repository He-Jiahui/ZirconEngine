use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::script::{VmError, VmStateBlob, VmStateSchema, VM_STATE_SCHEMA_VERSION_V3};
use zr_vm_rust_binding as zrvm;
use zr_vm_rust_binding_sys::ZrRustBindingStatus;

use super::super::errors::map_zr_error;
use super::super::values::from_zr_return_value_for_export;

#[derive(Clone, Copy)]
pub(crate) enum ZrVmStateEncoding {
    JsonV3,
    BytesV1,
}

impl ZrVmStateEncoding {
    pub(in crate::real_backend) fn load(
        session: &mut zrvm::ProjectSession,
        entry_module: &str,
    ) -> Result<Self, VmError> {
        let value = match session.call_module_export(entry_module, "stateEncoding", &[]) {
            Ok(value) => value,
            Err(error) if error.status == ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_NOT_FOUND => {
                return Ok(Self::JsonV3);
            }
            Err(error) => return Err(map_zr_error(error)),
        };
        let encoding = value.as_string().map_err(map_zr_error)?;
        match encoding.as_str() {
            "vm-state-json-v3" => Ok(Self::JsonV3),
            "vm-state-bytes-v1" => Ok(Self::BytesV1),
            _ => Err(VmError::Operation(format!(
                "unsupported zr_vm stateEncoding {encoding:?}"
            ))),
        }
    }

    pub(super) fn decode_state(
        self,
        value: Option<&zrvm::Value>,
        entry_module: &str,
    ) -> Result<VmStateBlob, VmError> {
        match (self, value) {
            (Self::JsonV3, None) => Ok(VmStateBlob::default()),
            (Self::JsonV3, Some(value)) => match value.kind() {
                zrvm::ValueKind::String => {
                    VmStateBlob::from_json(&value.as_string().map_err(map_zr_error)?)
                        .map_err(Into::into)
                }
                zrvm::ValueKind::Null => Ok(VmStateBlob::default()),
                other => Err(VmError::Operation(format!(
                    "zr_vm JSON saveState returned unsupported value kind {other:?}"
                ))),
            },
            (Self::BytesV1, Some(value)) if value.kind() == zrvm::ValueKind::Array => {
                match from_zr_return_value_for_export(value, entry_module, "saveState")
                    .map_err(map_zr_error)?
                {
                    ScriptHostValue::Bytes(bytes) => Ok(VmStateBlob::from_payload(bytes)),
                    _ => unreachable!("Array<uint> lowering returns only bytes"),
                }
            }
            (Self::BytesV1, _) => Err(VmError::Operation(
                "vm-state-bytes-v1 requires saveState to return Array<uint>".into(),
            )),
        }
    }

    pub(super) fn restore_argument(self, state: &VmStateBlob) -> Result<zrvm::Value, VmError> {
        match self {
            Self::JsonV3 => zrvm::Value::new_string(&state.to_json()?).map_err(map_zr_error),
            Self::BytesV1 => {
                if state.schema_version != VM_STATE_SCHEMA_VERSION_V3 || !state.types.is_empty() {
                    return Err(VmError::Operation(
                        "vm-state-bytes-v1 requires an opaque schema-v3 VmStateBlob".into(),
                    ));
                }
                let mut array = zrvm::Value::new_array().map_err(map_zr_error)?;
                for byte in &state.payload {
                    let value = zrvm::Value::new_int(i64::from(*byte)).map_err(map_zr_error)?;
                    array.array_push(&value).map_err(map_zr_error)?;
                }
                Ok(array)
            }
        }
    }

    pub(super) fn validate_restore_result(
        self,
        result: Option<&zrvm::Value>,
    ) -> Result<(), VmError> {
        if let Self::BytesV1 = self {
            let Some(value) = result else {
                return Err(VmError::Operation(
                    "vm-state-bytes-v1 requires restoreState to return success code 0".into(),
                ));
            };
            if value.kind() != zrvm::ValueKind::Int || value.as_int().map_err(map_zr_error)? != 0 {
                return Err(VmError::Operation(
                    "vm-state-bytes-v1 requires restoreState to return success code 0".into(),
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate_schema(self, schema: &VmStateSchema) -> Result<(), VmError> {
        if let Self::BytesV1 = self {
            if schema.schema_version != VM_STATE_SCHEMA_VERSION_V3 || !schema.types.is_empty() {
                return Err(VmError::Operation(
                    "vm-state-bytes-v1 cannot advertise a reflected or non-v3 state schema".into(),
                ));
            }
        }
        Ok(())
    }
}
