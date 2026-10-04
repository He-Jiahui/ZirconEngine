use zircon_plugin_sdk::native::{
    self, NativePluginEntryPointV3, NativePluginEntryReportV3, NativePluginHostFunctionTableV3,
    NativePluginStatic,
};

static REPORT: NativePluginStatic<NativePluginEntryReportV3> =
    NativePluginStatic::new(NativePluginEntryReportV3 {
        layout_epoch: native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
        package_manifest_toml: b"\0".as_ptr().cast(),
        diagnostics: b"ready\0".as_ptr().cast(),
        negotiated_capabilities: b"\0".as_ptr().cast(),
        required_capabilities: b"\0".as_ptr().cast(),
        denied_capabilities: b"\0".as_ptr().cast(),
        behavior: std::ptr::null(),
        bridge_methods: std::ptr::null(),
    });

fn ready(_: *const NativePluginHostFunctionTableV3) {}
fn controlled_panic(_: *const NativePluginHostFunctionTableV3) {
    panic!("controlled entry hook regression");
}
static NORMAL: NativePluginEntryPointV3 =
    NativePluginEntryPointV3::new(&REPORT, &REPORT, &[], &[], Some(ready));
static PANICKING: NativePluginEntryPointV3 =
    NativePluginEntryPointV3::new(&REPORT, &REPORT, &[], &[], Some(controlled_panic));
fn controlled_expression_panic() -> &'static NativePluginEntryPointV3 {
    panic!("controlled entry expression regression");
}

// Downstream expansion exercises actual public helper visibility and both
// entry expression evaluation and hook execution inside the exported boundary.
zircon_plugin_sdk::export_native_plugin_entry_v3!(normal_runtime_entry, NORMAL);
zircon_plugin_sdk::export_native_plugin_entry_v3!(normal_editor_entry, NORMAL);
zircon_plugin_sdk::export_native_plugin_entry_v3!(panic_runtime_entry, PANICKING);
zircon_plugin_sdk::export_native_plugin_entry_v3!(panic_editor_entry, PANICKING);
zircon_plugin_sdk::export_native_plugin_entry_v3!(
    panic_expression_entry,
    controlled_expression_panic()
);

fn host() -> NativePluginHostFunctionTableV3 {
    NativePluginHostFunctionTableV3 {
        abi_version: native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        host_handle: 1,
        granted_capabilities: b"\0".as_ptr().cast(),
        host_abi_version: None,
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    }
}

#[test]
fn normal_runtime_and_editor_entry_preserve_report_and_abi() {
    let host = host();
    assert_eq!(normal_runtime_entry(&host), REPORT.as_ptr());
    assert_eq!(normal_editor_entry(&host), REPORT.as_ptr());
    assert_eq!(native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3, 3);
    assert_eq!(native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH, 5);
    assert_eq!(native::ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4, 4);
}

#[test]
fn controlled_runtime_editor_and_expression_panics_return_null() {
    assert!(
        cfg!(panic = "unwind"),
        "containment validation requires unwind profile; abort is not a skipped pass"
    );
    let host = host();
    assert!(panic_runtime_entry(&host).is_null());
    assert!(panic_editor_entry(&host).is_null());
    assert!(panic_expression_entry(&host).is_null());
    assert_eq!(normal_runtime_entry(&host), REPORT.as_ptr());
}
