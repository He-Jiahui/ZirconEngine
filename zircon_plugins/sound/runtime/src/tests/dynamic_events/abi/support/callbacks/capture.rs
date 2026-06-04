use zircon_runtime_interface::{
    ZrPluginEventCallbackRequestV1, ZrPluginEventCallbackResultV1, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
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
    assert_eq!(unsafe { request.namespace.as_slice() }, NAMESPACE);
    assert_eq!(unsafe { request.plugin_id.as_slice() }, PLUGIN_ID);
    assert_eq!(unsafe { request.handler_id.as_slice() }, HANDLER_ID);
    assert_eq!(unsafe { request.event_id.as_slice() }, EVENT_ID.as_bytes());
    assert_eq!(unsafe { request.source_path.as_slice() }, SOURCE_PATH);
    assert_eq!(request.time_seconds, 2.5);
    assert_eq!(
        unsafe { request.payload_schema.as_slice() },
        PAYLOAD_SCHEMA.as_bytes()
    );
    assert_eq!(unsafe { request.payload.as_slice() }, PAYLOAD);
    unsafe {
        *result = ZrPluginEventCallbackResultV1::ok(ZIRCON_RUNTIME_ABI_VERSION_V1);
    }
    ZrStatus::ok()
}
