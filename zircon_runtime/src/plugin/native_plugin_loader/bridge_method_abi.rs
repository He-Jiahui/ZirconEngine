use super::abi_declarations::{
    NativePluginBridgeMethodTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
};
use super::bridge_method_bindings::{NativeBridgeMethodBinding, NativeBridgeMethodFn};
use super::native_strings::read_required_c_string;

pub(super) unsafe fn bridge_method_bindings_from_abi_v3(
    table: *const NativePluginBridgeMethodTableV3,
) -> Result<Vec<NativeBridgeMethodBinding>, String> {
    if table.is_null() {
        return Ok(Vec::new());
    }
    let table = unsafe { &*table };
    if table.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        return Err(format!(
            "unsupported native bridge method table ABI version {}; expected {}",
            table.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
        ));
    }
    if table.methods.is_null() {
        if table.method_count == 0 {
            return Ok(Vec::new());
        }
        return Err(
            "native bridge method table declared methods but methods pointer was null".to_string(),
        );
    }

    let methods = unsafe { std::slice::from_raw_parts(table.methods, table.method_count) };
    let mut bindings = Vec::with_capacity(methods.len());
    for method in methods {
        let interface_id = unsafe { read_required_c_string(method.interface_id, "interface_id")? };
        let method_name = unsafe { read_required_c_string(method.method_name, "method_name")? };
        let Some(callback) = method.method else {
            return Err(format!(
                "native bridge method `{interface_id}.{method_name}` declared no callback"
            ));
        };
        bindings.push(NativeBridgeMethodBinding::new(
            interface_id,
            method_name,
            NativeBridgeMethodFn::from_abi_v3(callback, method.user_data),
        ));
    }
    Ok(bindings)
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

    unsafe extern "C" fn test_bridge_method(call: NativePluginBridgeMethodCallV3) -> ZrStatus {
        if call.interface_slot == 3 && call.method_slot == 7 && call.user_data == 42 {
            ZrStatus::new(ZrStatusCode::CapabilityDenied, ZrByteSlice::empty())
        } else {
            ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
        }
    }
}
