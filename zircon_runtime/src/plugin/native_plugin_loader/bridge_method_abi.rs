use super::abi_declarations::{
    NativePluginBridgeMethodTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
};
use super::bridge_method_bindings::{NativeBridgeMethodBinding, NativeBridgeMethodFn};
use super::native_strings::read_required_c_string;

pub(super) type NativeBridgeMethodAbiResult<T> = std::result::Result<T, NativeBridgeMethodAbiError>;

#[derive(Debug)]
pub(super) enum NativeBridgeMethodAbiError {
    UnsupportedTableAbiVersion {
        actual: u32,
        expected: u32,
    },
    MissingMethodsPointerWithCount {
        method_count: usize,
    },
    InvalidRequiredField {
        field_name: &'static str,
        source: String,
    },
    MissingCallback {
        interface_id: String,
        method_name: String,
    },
}

impl std::fmt::Display for NativeBridgeMethodAbiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedTableAbiVersion { actual, expected } => write!(
                formatter,
                "unsupported native bridge method table ABI version {actual}; expected {expected}"
            ),
            Self::MissingMethodsPointerWithCount { .. } => formatter.write_str(
                "native bridge method table declared methods but methods pointer was null",
            ),
            Self::InvalidRequiredField { source, .. } => formatter.write_str(source),
            Self::MissingCallback {
                interface_id,
                method_name,
            } => write!(
                formatter,
                "native bridge method `{interface_id}.{method_name}` declared no callback"
            ),
        }
    }
}

impl std::error::Error for NativeBridgeMethodAbiError {}

pub(super) unsafe fn bridge_method_bindings_from_abi_v3(
    table: *const NativePluginBridgeMethodTableV3,
) -> NativeBridgeMethodAbiResult<Vec<NativeBridgeMethodBinding>> {
    if table.is_null() {
        return Ok(Vec::new());
    }
    let table = unsafe { &*table };
    if table.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        return Err(NativeBridgeMethodAbiError::UnsupportedTableAbiVersion {
            actual: table.abi_version,
            expected: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        });
    }
    if table.methods.is_null() {
        if table.method_count == 0 {
            return Ok(Vec::new());
        }
        return Err(NativeBridgeMethodAbiError::MissingMethodsPointerWithCount {
            method_count: table.method_count,
        });
    }

    let methods = unsafe { std::slice::from_raw_parts(table.methods, table.method_count) };
    let mut bindings = Vec::with_capacity(methods.len());
    for method in methods {
        let interface_id =
            unsafe { required_bridge_method_field(method.interface_id, "interface_id")? };
        let method_name =
            unsafe { required_bridge_method_field(method.method_name, "method_name")? };
        let Some(callback) = method.method else {
            return Err(NativeBridgeMethodAbiError::MissingCallback {
                interface_id,
                method_name,
            });
        };
        bindings.push(NativeBridgeMethodBinding::new(
            interface_id,
            method_name,
            NativeBridgeMethodFn::from_abi_v3(callback, method.user_data),
        ));
    }
    Ok(bindings)
}

unsafe fn required_bridge_method_field(
    value: *const std::ffi::c_char,
    field_name: &'static str,
) -> NativeBridgeMethodAbiResult<String> {
    unsafe { read_required_c_string(value, field_name) }.map_err(|source| {
        NativeBridgeMethodAbiError::InvalidRequiredField {
            field_name,
            source: source.to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus, ZrStatusCode};

    use super::super::abi_declarations::{
        NativePluginBridgeMethodCallV3, NativePluginBridgeMethodTableV3,
        NativePluginBridgeMethodV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    };
    use super::super::bridge_method_bindings::NativeBridgeCall;
    use super::*;

    #[test]
    fn bridge_method_bindings_parse_abi_v3_callback_table() {
        let interface_id = b"native.bridge.v1\0";
        let method_name = b"sample\0";
        let methods = [NativePluginBridgeMethodV3 {
            interface_id: interface_id.as_ptr().cast(),
            method_name: method_name.as_ptr().cast(),
            method: Some(test_bridge_method),
            user_data: 42,
        }];
        let table = NativePluginBridgeMethodTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            methods: methods.as_ptr(),
            method_count: methods.len(),
        };

        let bindings = unsafe { bridge_method_bindings_from_abi_v3(&table) }
            .expect("ABI bridge method table should parse");

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].interface_id(), "native.bridge.v1");
        assert_eq!(bindings[0].method_name(), "sample");
        let status = bindings[0].method.call(NativeBridgeCall {
            interface_slot: 3,
            method_slot: 7,
            payload: ZrByteSlice::empty(),
            output: ZrByteBufferRef::empty(),
        });
        assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
    }

    #[test]
    fn bridge_method_bindings_report_unsupported_table_abi_with_typed_error() {
        let table = NativePluginBridgeMethodTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 + 1,
            methods: std::ptr::null(),
            method_count: 0,
        };

        let error = unsafe { bridge_method_bindings_from_abi_v3(&table) }
            .expect_err("unsupported bridge method table ABI should be typed");

        assert!(matches!(
            error,
            NativeBridgeMethodAbiError::UnsupportedTableAbiVersion { actual, expected }
                if actual == ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 + 1
                    && expected == ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
        ));
    }

    #[test]
    fn bridge_method_typed_error_preserves_missing_callback_message() {
        let error = NativeBridgeMethodAbiError::MissingCallback {
            interface_id: "native.bridge.v1".to_string(),
            method_name: "sample".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "native bridge method `native.bridge.v1.sample` declared no callback"
        );
    }

    unsafe extern "C" fn test_bridge_method(call: NativePluginBridgeMethodCallV3) -> ZrStatus {
        if call.interface_slot == 3 && call.method_slot == 7 && call.user_data == 42 {
            ZrStatus::new(ZrStatusCode::CapabilityDenied, ZrByteSlice::empty())
        } else {
            ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
        }
    }
}
