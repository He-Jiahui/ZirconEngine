use zircon_runtime_interface::{ZrHostApiV1, ZrRuntimeApiV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::session::{capture_frame, create_session, destroy_session, handle_event};

static RUNTIME_API_V1: ZrRuntimeApiV1 = ZrRuntimeApiV1 {
    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
    size_bytes: core::mem::size_of::<ZrRuntimeApiV1>(),
    create_session: Some(create_session),
    destroy_session: Some(destroy_session),
    handle_event: Some(handle_event),
    capture_frame: Some(capture_frame),
};

#[no_mangle]
pub unsafe extern "C" fn zircon_runtime_get_api_v1(
    host: *const ZrHostApiV1,
) -> *const ZrRuntimeApiV1 {
    if !host.is_null() {
        let host = unsafe { &*host };
        if host.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return core::ptr::null();
        }
    }
    &RUNTIME_API_V1
}
