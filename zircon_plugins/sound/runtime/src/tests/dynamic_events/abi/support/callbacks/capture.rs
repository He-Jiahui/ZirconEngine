use zircon_runtime_interface::{
    ZrPluginEventCallbackRequestV1, ZrPluginEventCallbackResultV1, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1,
};

use super::super::detail::{
    EVENT_ID, HANDLER_ID, NAMESPACE, PAYLOAD, PAYLOAD_SCHEMA, PLUGIN_ID, SOURCE_PATH,
};

pub(crate) unsafe extern "C" fn capture_abi_callback(
    request: ZrPluginEventCallbackRequestV1,
    result: *mut ZrPluginEventCallbackResultV1,
) -> ZrStatus {
    assert!(!result.is_null());
    assert_eq!(request.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(
        unsafe {
            request
                .namespace
                .checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
        }
        .unwrap(),
        NAMESPACE
    );
    assert_eq!(
        unsafe {
            request
                .plugin_id
                .checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
        }
        .unwrap(),
        PLUGIN_ID
    );
    assert_eq!(
        unsafe {
            request
                .handler_id
                .checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
        }
        .unwrap(),
        HANDLER_ID
    );
    assert_eq!(
        unsafe {
            request
                .event_id
                .checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
        }
        .unwrap(),
        EVENT_ID.as_bytes()
    );
    assert_eq!(
        unsafe {
            request
                .source_path
                .checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
        }
        .unwrap(),
        SOURCE_PATH
    );
    assert_eq!(request.time_seconds, 2.5);
    assert_eq!(
        unsafe {
            request
                .payload_schema
                .checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
        }
        .unwrap(),
        PAYLOAD_SCHEMA.as_bytes()
    );
    assert_eq!(
        unsafe {
            request
                .payload
                .checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
        }
        .unwrap(),
        PAYLOAD
    );
    unsafe {
        *result = ZrPluginEventCallbackResultV1::ok(ZIRCON_RUNTIME_ABI_VERSION_V1);
    }
    ZrStatus::ok()
}
