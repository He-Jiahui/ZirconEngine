use std::ffi::CStr;

use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus, ZrStatusCode};

use crate::native::{
    self, NativePluginBridgeMethodCallV3, NativePluginByteSliceV3, NativePluginCallbackStatusV3,
    NativePluginHostFunctionTableV3, NativePluginOutputSinkV4, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
};

const PLUGIN_ID: &[u8] = b"dist_helper_fixture\0";
const PACKAGE_MANIFEST: &str = "id = \"dist_helper_fixture\"\nversion = \"0.1.0\"\n\0";
const RUNTIME_ENTRY: &[u8] = b"dist_helper_runtime_entry_v3\0";
const EDITOR_ENTRY: &[u8] = b"dist_helper_editor_entry_v3\0";
const REQUESTED_CAPABILITIES: &[u8] = b"runtime.plugin.dist_helper\neditor.extension.dist_helper\0";
const RUNTIME_NEGOTIATED: &[u8] = b"runtime.plugin.dist_helper\0";
const EDITOR_NEGOTIATED: &[u8] = b"editor.extension.dist_helper\0";
const READY: &[u8] = b"ready\0";
const MISSING: &[u8] = b"missing\0";
const EMPTY: &[u8] = b"\0";
const EMPTY_COMMANDS_V4: &[u8] =
    b"schema = \"zircon.native.command-manifest/4\"\ncommands = []\n\0";
const RUNTIME_INTERFACE: &[u8] = b"dist_helper.runtime\0";
const RUNTIME_METHOD: &[u8] = b"tick\0";

crate::native_dist_plugin_v3! {
    plugin_id: PLUGIN_ID,
    package_manifest: PACKAGE_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    runtime_entry: dist_helper_runtime_entry_v3,
    runtime_entry_name: RUNTIME_ENTRY,
    editor_entry: dist_helper_editor_entry_v3,
    editor_entry_name: EDITOR_ENTRY,
    requested_capabilities: REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING,
    runtime: {
        required_capabilities: ["runtime.plugin.dist_helper"],
        denied_capabilities: ["runtime.plugin.denied_dist_helper"],
        negotiated_capabilities: RUNTIME_NEGOTIATED,
        diagnostics: READY,
        is_stateless: false,
        state_schema_version: 3,
        command_manifest_schema: Some(native::NATIVE_COMMAND_MANIFEST_SCHEMA_V4),
        event_manifest_schema: Some(native::NATIVE_EVENT_MANIFEST_SCHEMA_V3),
        registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EMPTY_COMMANDS_V4),
        event_manifest: Some(EMPTY),
        registration_manifest: Some(EMPTY),
        invoke_command: Some(dist_helper_invoke_command),
        save_state: None,
        restore_state: None,
        unload: None,
        bridge_methods: [
            {
                interface: RUNTIME_INTERFACE,
                method: RUNTIME_METHOD,
                function: dist_helper_tick,
                user_data: 99,
            },
        ],
        on_host_ready: None,
    },
    editor: {
        required_capabilities: ["editor.extension.dist_helper"],
        denied_capabilities: [],
        negotiated_capabilities: EDITOR_NEGOTIATED,
        diagnostics: READY,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: Some(EMPTY_COMMANDS_V4),
        event_manifest: Some(EMPTY),
        registration_manifest: None,
        invoke_command: Some(dist_helper_invoke_command),
        save_state: None,
        restore_state: None,
        unload: None,
        bridge_methods: [],
        on_host_ready: None,
    },
}

// 本夹具只验证导出符号和静态表连接；其有状态配置缺少保存与恢复回调，不能代表宿主加载验收。
#[test]
fn dist_plugin_one_file_export_compiles() {
    let descriptor = zircon_native_plugin_descriptor_v3();
    assert!(!descriptor.is_null());
    let descriptor = unsafe { &*descriptor };
    assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
    assert_eq!(
        unsafe { CStr::from_ptr(descriptor.plugin_id) },
        CStr::from_bytes_with_nul(PLUGIN_ID).expect("plugin id is nul terminated")
    );

    let granted = b"runtime.plugin.dist_helper\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 7,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    let report = dist_helper_runtime_entry_v3(&host);
    assert!(!report.is_null());
    let report = unsafe { &*report };
    assert!(!report.behavior.is_null());
    assert!(!report.bridge_methods.is_null());

    let behavior = unsafe { &*report.behavior };
    assert!(!behavior.registration_manifest.is_null());
    let bridge_methods = unsafe { &*report.bridge_methods };
    assert_eq!(bridge_methods.method_count, 1);
    assert!(!bridge_methods.methods.is_null());

    let bridge_method = unsafe { &*bridge_methods.methods };
    assert_eq!(bridge_method.user_data, 99);
    let call = NativePluginBridgeMethodCallV3 {
        interface_slot: 0,
        method_slot: 0,
        payload: ZrByteSlice::empty(),
        output: ZrByteBufferRef::empty(),
        user_data: bridge_method.user_data,
    };
    let status = unsafe { bridge_method.method.expect("bridge method is present")(call) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok);
}

unsafe extern "C" fn dist_helper_invoke_command(
    _command_slot: u32,
    _payload: NativePluginByteSliceV3,
    _output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV3 {
    native::callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, MISSING)
}

unsafe extern "C" fn dist_helper_tick(_call: NativePluginBridgeMethodCallV3) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}
