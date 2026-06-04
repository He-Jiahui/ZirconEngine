use zircon_runtime_interface::{
    ZrByteSlice, ZrPluginEventCallbackRequestV1, ZrPluginEventCallbackResultV1, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::detail::FAILURE_DETAIL;

pub(crate) unsafe extern "C" fn failing_abi_callback(
    _request: ZrPluginEventCallbackRequestV1,
    result: *mut ZrPluginEventCallbackResultV1,
) -> ZrStatus {
    unsafe {
        *result = ZrPluginEventCallbackResultV1::failed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrByteSlice::from_static(FAILURE_DETAIL),
        );
    }
    ZrStatus::ok()
}
