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

#[test]
fn native_ffi_panic_guards_return_when_both_payload_destructors_panic() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SECONDARY_DROPS: AtomicUsize = AtomicUsize::new(0);

    struct SecondaryPayload;
    impl Drop for SecondaryPayload {
        fn drop(&mut self) {
            SECONDARY_DROPS.fetch_add(1, Ordering::SeqCst);
            panic!("secondary payload destructor");
        }
    }
    struct PrimaryPayload;
    impl Drop for PrimaryPayload {
        fn drop(&mut self) {
            std::panic::panic_any(SecondaryPayload);
        }
    }
    SECONDARY_DROPS.store(0, Ordering::SeqCst);
    assert_eq!(
        catch_native_host_api_panic(|| std::panic::panic_any(PrimaryPayload)).status_code(),
        ZrStatusCode::Panic
    );
    assert_eq!(SECONDARY_DROPS.load(Ordering::SeqCst), 0);

    assert_eq!(
        catch_native_plugin_host_callback_panic(|| std::panic::panic_any(PrimaryPayload)),
        ZIRCON_NATIVE_PLUGIN_STATUS_PANIC
    );
    assert_eq!(SECONDARY_DROPS.load(Ordering::SeqCst), 0);

    assert_eq!(
        catch_native_plugin_output_sink_panic(|| std::panic::panic_any(PrimaryPayload)).code,
        ZIRCON_NATIVE_PLUGIN_STATUS_PANIC
    );
    assert_eq!(SECONDARY_DROPS.load(Ordering::SeqCst), 0);
}
