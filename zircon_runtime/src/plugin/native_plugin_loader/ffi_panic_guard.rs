use std::panic::{catch_unwind, AssertUnwindSafe};

use zircon_runtime_interface::{ZrByteSlice, ZrStatus, ZrStatusCode};

use super::abi_declarations::ZIRCON_NATIVE_PLUGIN_STATUS_PANIC;

pub(super) fn catch_native_host_api_panic(call: impl FnOnce() -> ZrStatus) -> ZrStatus {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(status) => status,
        Err(_) => ZrStatus::new(
            ZrStatusCode::Panic,
            ZrByteSlice::from_static(b"native host API panic caught at FFI boundary"),
        ),
    }
}

pub(super) fn catch_native_plugin_host_callback_panic(call: impl FnOnce() -> u32) -> u32 {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(status) => status,
        Err(_) => ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_host_api_panic_guard_returns_ffi_panic_status() {
        let status = catch_native_host_api_panic(|| panic!("native host API panic"));

        assert_eq!(status.status_code(), ZrStatusCode::Panic);
    }

    #[test]
    fn native_plugin_host_callback_panic_guard_returns_abi_panic_status() {
        let status =
            catch_native_plugin_host_callback_panic(|| panic!("native plugin host callback panic"));

        assert_eq!(status, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC);
    }
}
