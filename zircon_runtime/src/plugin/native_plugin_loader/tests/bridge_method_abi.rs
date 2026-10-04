use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus, ZrStatusCode};

use super::super::abi_declarations::{
    NativePluginBridgeMethodCallV3, NativePluginBridgeMethodTableV3, NativePluginBridgeMethodV3,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
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
