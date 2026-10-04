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
