use std::ffi::CStr;
use std::panic::{catch_unwind, AssertUnwindSafe};

use zircon_runtime_interface::{ZrByteSlice, ZrStatus, ZrStatusCode};

use super::abi_declarations::{NativePluginCallbackStatusV3, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC};

pub(super) const NATIVE_PLUGIN_OUTPUT_SINK_PANIC_DIAGNOSTIC: &CStr =
    c"native plugin output sink panic caught at FFI boundary";

pub(super) fn catch_native_host_api_panic(call: impl FnOnce() -> ZrStatus) -> ZrStatus {
    catch_native_ffi_panic(call, || {
        ZrStatus::new(
            ZrStatusCode::Panic,
            ZrByteSlice::from_static(b"native host API panic caught at FFI boundary"),
        )
    })
}

pub(super) fn catch_native_plugin_host_callback_panic(call: impl FnOnce() -> u32) -> u32 {
    catch_native_ffi_panic(call, || ZIRCON_NATIVE_PLUGIN_STATUS_PANIC)
}

pub(super) fn catch_native_plugin_output_sink_panic(
    call: impl FnOnce() -> NativePluginCallbackStatusV3,
) -> NativePluginCallbackStatusV3 {
    catch_native_ffi_panic(call, || NativePluginCallbackStatusV3 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
        diagnostics: NATIVE_PLUGIN_OUTPUT_SINK_PANIC_DIAGNOSTIC.as_ptr(),
    })
}

fn catch_native_ffi_panic<Status>(
    call: impl FnOnce() -> Status,
    panic_status: impl FnOnce() -> Status,
) -> Status {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(status) => status,
        Err(_) => panic_status(),
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

    #[test]
    fn native_plugin_output_sink_panic_guard_returns_typed_callback_status() {
        let status =
            catch_native_plugin_output_sink_panic(|| panic!("native plugin output sink panic"));

        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC);
        assert!(!status.diagnostics.is_null());
    }
}
