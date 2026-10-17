use std::sync::atomic::{AtomicUsize, Ordering};

use zircon_plugin_sdk::native::{
    self, NativePluginEntryPointV3, NativePluginEntryReportV3, NativePluginHostFunctionTableV3,
    NativePluginStatic,
};

static SECONDARY_DROPS: AtomicUsize = AtomicUsize::new(0);

struct SecondaryPayload;

impl Drop for SecondaryPayload {
    fn drop(&mut self) {
        SECONDARY_DROPS.fetch_add(1, Ordering::SeqCst);
        panic!("secondary panic payload destructor");
    }
}

struct PrimaryPayload;

impl Drop for PrimaryPayload {
    fn drop(&mut self) {
        std::panic::panic_any(SecondaryPayload);
    }
}

static REPORT: NativePluginStatic<NativePluginEntryReportV3> =
    NativePluginStatic::new(NativePluginEntryReportV3 {
        layout_epoch: native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
        package_manifest_toml: c"".as_ptr(),
        diagnostics: c"ready".as_ptr(),
        negotiated_capabilities: c"".as_ptr(),
        required_capabilities: c"".as_ptr(),
        denied_capabilities: c"".as_ptr(),
        behavior: std::ptr::null(),
        bridge_methods: std::ptr::null(),
    });

fn panic_in_entry_hook(_: *const NativePluginHostFunctionTableV3) {
    std::panic::panic_any(PrimaryPayload);
}

static PANICKING: NativePluginEntryPointV3 =
    NativePluginEntryPointV3::new(&REPORT, &REPORT, &[], &[], Some(panic_in_entry_hook));

fn panic_in_entry_expression() -> &'static NativePluginEntryPointV3 {
    std::panic::panic_any(PrimaryPayload);
}

zircon_plugin_sdk::export_native_plugin_entry_v3!(panic_runtime_entry, PANICKING);
zircon_plugin_sdk::export_native_plugin_entry_v3!(panic_editor_entry, PANICKING);
zircon_plugin_sdk::export_native_plugin_entry_v3!(
    panic_expression_entry,
    panic_in_entry_expression()
);

fn host() -> NativePluginHostFunctionTableV3 {
    NativePluginHostFunctionTableV3 {
        abi_version: native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        host_handle: 1,
        granted_capabilities: c"".as_ptr(),
        host_abi_version: None,
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    }
}

fn main() {
    const {
        assert!(
            cfg!(panic = "unwind"),
            "shipping panic guard fixture requires panic=unwind"
        );
    }

    SECONDARY_DROPS.store(0, Ordering::SeqCst);
    let callback_status =
        native::catch_native_callback_panic(c"callback panic".to_bytes_with_nul(), || {
            std::panic::panic_any(PrimaryPayload)
        });
    assert_eq!(
        callback_status.code,
        native::ZIRCON_NATIVE_PLUGIN_STATUS_PANIC
    );
    assert_eq!(SECONDARY_DROPS.load(Ordering::SeqCst), 0);

    let host = host();
    for entry in [
        panic_runtime_entry,
        panic_editor_entry,
        panic_expression_entry,
    ] {
        SECONDARY_DROPS.store(0, Ordering::SeqCst);
        assert!(unsafe { entry(&host) }.is_null());
        assert_eq!(SECONDARY_DROPS.load(Ordering::SeqCst), 0);
    }
}
