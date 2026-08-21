use serde::{Deserialize, Serialize};
#[cfg(feature = "abi_v2_only")]
use std::ffi::c_char;
use std::io::{self, Write};
use zircon_plugin_sdk::native::{
    self, NativePluginBridgeMethodCallV3, NativePluginByteSliceV2, NativePluginCallbackStatusV2,
    NativePluginHostFunctionTableV3, NativePluginOutputSinkV4, NativePluginOwnedByteBufferV2,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK, bytes_from_slice, callback_status as status, owned_bytes,
};

#[cfg(feature = "abi_unknown_version")]
const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_ABI_VERSION: u32 = 99;
#[cfg(not(feature = "abi_unknown_version"))]
const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_ABI_VERSION: u32 =
    zircon_plugin_sdk::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION;
const IMPORT_REQUEST_MAGIC: &[u8] = b"ZRIMP001\n";
const IMPORT_RESPONSE_MAGIC: &[u8] = b"ZRIMO001\n";
const IMPORT_ENVELOPE_LENGTH_BYTES: usize = std::mem::size_of::<u64>();
const MAX_IMPORT_METADATA_BYTES: usize = 64 * 1024;
const MAX_IMPORT_SOURCE_BYTES: usize = 256 * 1024;

#[cfg(feature = "abi_v2_only")]
const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2: u32 = 2;

zircon_plugin_sdk::declare_plugin! {
    NATIVE_DYNAMIC_FIXTURE_DECLARATION {
        id: PLUGIN_ID = "native_dynamic_fixture",
        display_name: "Native Dynamic Fixture",
        category: sdk,
        module: MODULE_NAME = "native_dynamic_fixture.runtime",
        crate_name: NATIVE_CRATE_NAME = "zircon_plugin_native_dynamic_fixture_native",
        module_description: "Real dynamic library fixture for ABI v3 native plugin loading",
        targets: [client_runtime, server_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.native_dynamic_fixture" => runtime_registration,
            IMPORTER_CAPABILITY = "runtime.asset.importer.native_dynamic_fixture.data_json" => runtime_registration,
            EDITOR_CAPABILITY = "editor.extension.native_dynamic_fixture" => editor_registration,
        ],
        maturity: experimental,
        packaging: [native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_native_dynamic_fixture_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [{
                    id: "native_dynamic_fixture.runtime_tick",
                    module: "runtime",
                    stage: "Update",
                    order: 0,
                    sets: ["native_dynamic_fixture"],
                    access: ["write:world"],
                    thread_affinity: "main-thread-only",
                    bridge_interface: "native_dynamic_fixture.runtime",
                    bridge_method: "tick",
                }],
                events: [{
                    namespace: "native_dynamic_fixture",
                    name: "echoed",
                    stable_hash: 0,
                    schema: "bytes",
                }],
                extensions: [{
                    point: "runtime.asset.importer.data",
                    contribution: "plugin.native_dynamic_fixture.data_json",
                    schema: "zircon.runtime.asset-importer.data/1",
                }],
            },
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_native_dynamic_fixture_editor_entry_v3",
                registration_manifest: NATIVE_EDITOR_REGISTRATION_MANIFEST,
                modules: [{ name: "editor", kind: "editor" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

const FIXTURE_DATA_IMPORTER_ID: &str = "native_dynamic_fixture.data_json";

#[cfg(feature = "abi_v2_only")]
const V2_RUNTIME_ENTRY: &[u8] = b"zircon_native_dynamic_fixture_runtime_entry_v2\0";
#[cfg(feature = "abi_v2_only")]
const V2_RUNTIME_DIAGNOSTICS: &[u8] =
    b"runtime v2 entry reached with host ABI table\nnegotiated runtime.plugin.native_dynamic_fixture\0";
#[cfg(feature = "abi_v2_only")]
const V2_MISSING_HOST_DIAGNOSTICS: &[u8] = b"native v2 entry missing negotiated host ABI table\0";

#[cfg(feature = "abi_v2_only")]
#[repr(C)]
struct NativePluginAbiV2 {
    abi_version: u32,
    plugin_id: *const c_char,
    package_manifest_toml: *const c_char,
    runtime_entry_name: *const c_char,
    editor_entry_name: *const c_char,
    requested_capabilities: *const c_char,
}

#[cfg(feature = "abi_v2_only")]
#[repr(C)]
struct NativePluginEntryReportV2 {
    abi_version: u32,
    package_manifest_toml: *const c_char,
    diagnostics: *const c_char,
    negotiated_capabilities: *const c_char,
    behavior: *const NativePluginBehaviorV2,
}

#[cfg(feature = "abi_v2_only")]
#[repr(C)]
struct NativePluginHostFunctionTableV2 {
    abi_version: u32,
    host_handle: u64,
    granted_capabilities: *const c_char,
    host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    host_has_capability:
        Option<unsafe extern "C" fn(*const NativePluginHostFunctionTableV2, *const c_char) -> u32>,
}

#[cfg(feature = "abi_v2_only")]
#[repr(C)]
struct FixtureNativePluginByteSliceV2 {
    data: *const u8,
    len: usize,
}

#[cfg(feature = "abi_v2_only")]
#[repr(C)]
struct FixtureNativePluginOwnedByteBufferV2 {
    data: *mut u8,
    len: usize,
    capacity: usize,
    owner_token: u64,
    free: Option<FixtureNativePluginFreeBytesFnV2>,
}

#[cfg(feature = "abi_v2_only")]
#[repr(C)]
struct FixtureNativePluginCallbackStatusV2 {
    code: u32,
    diagnostics: *const c_char,
}

#[cfg(feature = "abi_v2_only")]
type FixtureNativePluginFreeBytesFnV2 = unsafe extern "C" fn(
    FixtureNativePluginOwnedByteBufferV2,
)
    -> FixtureNativePluginCallbackStatusV2;
#[cfg(feature = "abi_v2_only")]
type FixtureNativePluginInvokeCommandFnV2 =
    unsafe extern "C" fn(
        *const c_char,
        FixtureNativePluginByteSliceV2,
        *mut FixtureNativePluginOwnedByteBufferV2,
    ) -> FixtureNativePluginCallbackStatusV2;
#[cfg(feature = "abi_v2_only")]
type FixtureNativePluginSaveStateFnV2 = unsafe extern "C" fn(
    *mut FixtureNativePluginOwnedByteBufferV2,
)
    -> FixtureNativePluginCallbackStatusV2;
#[cfg(feature = "abi_v2_only")]
type FixtureNativePluginRestoreStateFnV2 =
    unsafe extern "C" fn(FixtureNativePluginByteSliceV2) -> FixtureNativePluginCallbackStatusV2;
#[cfg(feature = "abi_v2_only")]
type FixtureNativePluginUnloadFnV2 = unsafe extern "C" fn() -> FixtureNativePluginCallbackStatusV2;

#[cfg(feature = "abi_v2_only")]
#[repr(C)]
struct NativePluginBehaviorV2 {
    abi_version: u32,
    is_stateless: u32,
    command_manifest: *const c_char,
    event_manifest: *const c_char,
    invoke_command: Option<FixtureNativePluginInvokeCommandFnV2>,
    save_state: Option<FixtureNativePluginSaveStateFnV2>,
    restore_state: Option<FixtureNativePluginRestoreStateFnV2>,
    unload: Option<FixtureNativePluginUnloadFnV2>,
}

#[cfg(feature = "abi_v2_only")]
#[repr(transparent)]
struct SyncStatic<T>(T);

#[cfg(feature = "abi_v2_only")]
unsafe impl<T> Sync for SyncStatic<T> {}

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");
#[cfg(feature = "runtime_entry_export_missing")]
const fn fixture_runtime_entry() -> &'static [u8] {
    b"zircon_native_dynamic_fixture_runtime_entry_missing_v3\0"
}
#[cfg(not(feature = "runtime_entry_export_missing"))]
const fn fixture_runtime_entry() -> &'static [u8] {
    NATIVE_RUNTIME_ENTRY.cstr()
}
#[cfg(not(feature = "required_capability_missing"))]
const fn fixture_requested_capabilities() -> &'static [u8] {
    NATIVE_REQUESTED_CAPABILITIES
}
#[cfg(feature = "required_capability_missing")]
const fn fixture_requested_capabilities() -> &'static [u8] {
    b"runtime.asset.importer.native_dynamic_fixture.data_json\neditor.extension.native_dynamic_fixture\0"
}
const RUNTIME_NEGOTIATED_CAPABILITIES: &[u8] =
    b"runtime.plugin.native_dynamic_fixture\nruntime.asset.importer.native_dynamic_fixture.data_json\0";
const EDITOR_NEGOTIATED_CAPABILITIES: &[u8] = b"editor.extension.native_dynamic_fixture\0";
const EDITOR_DIAGNOSTICS_V3: &[u8] =
    b"editor entry reached with v3 host ABI table\nnegotiated editor.extension.native_dynamic_fixture\0";
const MISSING_HOST_DIAGNOSTICS_V3: &[u8] = b"native v3 entry missing negotiated host ABI table\0";
const RUNTIME_DIAGNOSTICS_WITH_DENIED_CAPABILITY_V3: &[u8] = b"runtime v3 entry reached with host ABI table\nnegotiated runtime.plugin.native_dynamic_fixture\ndenied capability runtime.plugin.denied_fixture\0";
const COMMAND_ECHO_SLOT: u32 = 0;
const COMMAND_BOUNDED_OVERFLOW_SLOT: u32 = 1;
const COMMAND_PANIC_SLOT: u32 = 2;
const COMMAND_ASSET_IMPORT_SLOT: u32 = 3;
const RUNTIME_COMMAND_MANIFEST_TEXT: &str = concat!(
    r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "echo"
slot = 0
payload_schema = "bytes"
max_output_bytes = 1048576
[[commands]]
name = "bounded_overflow"
slot = 1
payload_schema = "bytes"
max_output_bytes = 4
[[commands]]
name = "panic"
slot = 2
payload_schema = "bytes"
max_output_bytes = 0
[[commands]]
name = "asset.import/native_dynamic_fixture.data_json"
slot = 3
payload_schema = "ZRIMP001"
max_output_bytes = 1048576
"#,
    "\0"
);
const RUNTIME_COMMAND_MANIFEST: &[u8] = RUNTIME_COMMAND_MANIFEST_TEXT.as_bytes();
const RUNTIME_EVENT_MANIFEST: &[u8] = b"event=native_dynamic_fixture.echoed;payload=bytes\0";
const RUNTIME_BRIDGE_INTERFACE: &[u8] = b"native_dynamic_fixture.runtime\0";
const RUNTIME_BRIDGE_METHOD_TICK: &[u8] = b"tick\0";
const RUNTIME_HOST_LOG_TARGET: &[u8] = b"native_dynamic_fixture.runtime\0";
const EDITOR_HOST_LOG_TARGET: &[u8] = b"native_dynamic_fixture.editor\0";
const RUNTIME_HOST_LOG_MESSAGE: &[u8] = b"runtime v3 host log callback reached\0";
const EDITOR_HOST_LOG_MESSAGE: &[u8] = b"editor v3 host log callback reached\0";
const RUNTIME_HOST_DIAGNOSTIC_PATH: &[u8] = b"plugin.native_dynamic_fixture.runtime.entry\0";
const EDITOR_HOST_DIAGNOSTIC_PATH: &[u8] = b"plugin.native_dynamic_fixture.editor.entry\0";
const HOST_DIAGNOSTIC_UNIT: &[u8] = b"count\0";
const RUNTIME_HOST_DIAGNOSTIC_TAGS: &[u8] = b"plugin,native,runtime\0";
const EDITOR_HOST_DIAGNOSTIC_TAGS: &[u8] = b"plugin,native,editor\0";
const EDITOR_COMMAND_MANIFEST_TEXT: &str = concat!(
    r#"schema = "zircon.native.command-manifest/4"
commands = []
"#,
    "\0"
);
const EDITOR_COMMAND_MANIFEST: &[u8] = EDITOR_COMMAND_MANIFEST_TEXT.as_bytes();
const EDITOR_EVENT_MANIFEST: &[u8] = b"\0";
const STATUS_ECHO_DIAGNOSTICS: &[u8] = b"serialized command echo completed\0";
const STATUS_ASSET_IMPORT_DIAGNOSTICS: &[u8] = b"native fixture asset import completed\0";
const STATUS_ASSET_IMPORT_INVALID_DIAGNOSTICS: &[u8] =
    b"native fixture asset import request was malformed\0";
const STATUS_DENIED_COMMAND_DIAGNOSTICS: &[u8] = b"denied native command unknown\0";
const STATUS_PANIC_DIAGNOSTICS: &[u8] = b"native fixture caught panic during command invocation\0";
const STATUS_BAD_COMMAND_DIAGNOSTICS: &[u8] = b"native command slot was not declared\0";
const STATUS_BAD_OUTPUT_DIAGNOSTICS: &[u8] = b"native command output pointer was null\0";
const STATUS_STATE_SAVE_DIAGNOSTICS: &[u8] = b"state save completed\0";
const STATUS_STATE_RESTORE_DIAGNOSTICS: &[u8] = b"state restore accepted\0";
const STATUS_STATE_RESTORE_INVALID_DIAGNOSTICS: &[u8] = b"state restore rejected invalid blob\0";
const STATUS_UNLOAD_DIAGNOSTICS: &[u8] = b"unload callback reached\0";
const STATUS_STATELESS_UNLOAD_DIAGNOSTICS: &[u8] = b"stateless unload callback reached\0";
const STATUS_STATELESS_COMMAND_DENIED_DIAGNOSTICS: &[u8] =
    b"stateless editor command dispatch has no commands\0";
const RUNTIME_STATE_BLOB: &[u8] = b"state:v3:native_dynamic_fixture";

#[derive(Deserialize)]
struct NativeAssetImportRequestMetadata {
    importer_id: String,
    source_uri: String,
    source_path: String,
}

#[derive(Serialize)]
struct NativeAssetImportResponseMetadata<'a> {
    importer_id: &'a str,
    entries: [NativeAssetImportResponseEntry<'a>; 1],
}

#[derive(Serialize)]
struct NativeAssetImportResponseEntry<'a> {
    locator: &'a str,
    imported_asset: NativeAssetImportResponseAsset<'a>,
    migration_report: NativeAssetImportMigrationReport,
    diagnostics: [String; 1],
}

#[derive(Serialize)]
struct NativeAssetImportResponseAsset<'a> {
    #[serde(rename = "Data")]
    data: NativeAssetImportData<'a>,
}

#[derive(Serialize)]
struct NativeAssetImportData<'a> {
    uri: &'a str,
    format: &'static str,
    text: &'a str,
    canonical_json: serde_json::Value,
}

#[derive(Serialize)]
struct NativeAssetImportMigrationReport {
    source_schema_version: u32,
    target_schema_version: u32,
    summary: String,
}

struct BoundedResponseWriter {
    bytes: Vec<u8>,
    max_bytes: usize,
}

impl BoundedResponseWriter {
    fn new(max_bytes: usize, source_bytes: usize) -> Result<Self, String> {
        let header_bytes = IMPORT_RESPONSE_MAGIC.len() + IMPORT_ENVELOPE_LENGTH_BYTES;
        if max_bytes < header_bytes {
            return Err("native import response budget is smaller than its envelope".to_string());
        }
        let estimated_bytes = header_bytes
            .saturating_add(source_bytes.saturating_mul(2))
            .saturating_add(512)
            .min(max_bytes);
        let mut bytes = Vec::with_capacity(estimated_bytes);
        bytes.extend_from_slice(IMPORT_RESPONSE_MAGIC);
        bytes.extend_from_slice(&[0; IMPORT_ENVELOPE_LENGTH_BYTES]);
        Ok(Self { bytes, max_bytes })
    }

    fn finish(mut self) -> Result<Vec<u8>, String> {
        let metadata_start = IMPORT_RESPONSE_MAGIC.len() + IMPORT_ENVELOPE_LENGTH_BYTES;
        let metadata_len = self
            .bytes
            .len()
            .checked_sub(metadata_start)
            .ok_or_else(|| "native import response envelope was truncated".to_string())?;
        let metadata_len = u64::try_from(metadata_len)
            .map_err(|_| "native import response metadata length exceeds u64".to_string())?;
        let length_start = IMPORT_RESPONSE_MAGIC.len();
        self.bytes[length_start..metadata_start].copy_from_slice(&metadata_len.to_le_bytes());
        Ok(self.bytes)
    }
}

impl Write for BoundedResponseWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let new_len = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("native import response length overflow"))?;
        if new_len > self.max_bytes {
            return Err(io::Error::other(
                "native import response exceeds the host output budget",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(all(
    not(feature = "descriptor_export_missing"),
    not(feature = "abi_v2_only")
))]
zircon_plugin_sdk::native_dist_plugin_v3! {
    plugin_id: NATIVE_PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_ABI_VERSION,
    runtime_entry: zircon_native_dynamic_fixture_runtime_entry_v3,
    runtime_entry_name: fixture_runtime_entry(),
    editor_entry: zircon_native_dynamic_fixture_editor_entry_v3,
    editor_entry_name: NATIVE_EDITOR_ENTRY.cstr(),
    requested_capabilities: fixture_requested_capabilities(),
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS_V3,
    runtime: {
        required_capabilities: ["runtime.plugin.native_dynamic_fixture"],
        denied_capabilities: ["runtime.plugin.denied_fixture"],
        negotiated_capabilities: RUNTIME_NEGOTIATED_CAPABILITIES,
        diagnostics: RUNTIME_DIAGNOSTICS_WITH_DENIED_CAPABILITY_V3,
        is_stateless: false,
        state_schema_version: 3,
        command_manifest_schema: Some(native::NATIVE_COMMAND_MANIFEST_SCHEMA_V4),
        event_manifest_schema: Some(native::NATIVE_EVENT_MANIFEST_SCHEMA_V3),
        registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(RUNTIME_COMMAND_MANIFEST),
        event_manifest: Some(RUNTIME_EVENT_MANIFEST),
        registration_manifest: Some(NATIVE_RUNTIME_REGISTRATION_MANIFEST),
        invoke_command: Some(fixture_invoke_command),
        save_state: Some(fixture_save_state),
        restore_state: Some(fixture_restore_state),
        unload: Some(fixture_unload),
        bridge_methods: [
            {
                interface: RUNTIME_BRIDGE_INTERFACE,
                method: RUNTIME_BRIDGE_METHOD_TICK,
                function: fixture_runtime_tick_bridge,
                user_data: 0,
            },
        ],
        on_host_ready: Some(emit_host_v3_runtime_signals),
    },
    editor: {
        required_capabilities: ["editor.extension.native_dynamic_fixture"],
        denied_capabilities: [],
        negotiated_capabilities: EDITOR_NEGOTIATED_CAPABILITIES,
        diagnostics: EDITOR_DIAGNOSTICS_V3,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: Some(native::NATIVE_COMMAND_MANIFEST_SCHEMA_V4),
        event_manifest_schema: None,
        registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EDITOR_COMMAND_MANIFEST),
        event_manifest: Some(EDITOR_EVENT_MANIFEST),
        registration_manifest: Some(NATIVE_EDITOR_REGISTRATION_MANIFEST),
        invoke_command: Some(fixture_stateless_invoke_command),
        save_state: None,
        restore_state: None,
        unload: Some(fixture_stateless_unload),
        bridge_methods: [],
        on_host_ready: Some(emit_host_v3_editor_signals),
    },
}

// ABI v2 stays descriptor-compatible for the plan's fallback coverage, but its
// behavior callbacks are intentionally not reintroduced after the V4 hard cut.
#[cfg(feature = "abi_v2_only")]
static V2_RUNTIME_BEHAVIOR: SyncStatic<NativePluginBehaviorV2> =
    SyncStatic(NativePluginBehaviorV2 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2,
        is_stateless: 0,
        command_manifest: std::ptr::null(),
        event_manifest: std::ptr::null(),
        invoke_command: None,
        save_state: None,
        restore_state: None,
        unload: None,
    });

#[cfg(feature = "abi_v2_only")]
static V2_DESCRIPTOR: SyncStatic<NativePluginAbiV2> = SyncStatic(NativePluginAbiV2 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2,
    plugin_id: NATIVE_PLUGIN_ID.as_ptr().cast(),
    package_manifest_toml: PLUGIN_MANIFEST.as_ptr().cast(),
    runtime_entry_name: V2_RUNTIME_ENTRY.as_ptr().cast(),
    editor_entry_name: std::ptr::null(),
    requested_capabilities: NATIVE_REQUESTED_CAPABILITIES.as_ptr().cast(),
});

#[cfg(feature = "abi_v2_only")]
static V2_RUNTIME_REPORT: SyncStatic<NativePluginEntryReportV2> =
    SyncStatic(NativePluginEntryReportV2 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2,
        package_manifest_toml: PLUGIN_MANIFEST.as_ptr().cast(),
        diagnostics: V2_RUNTIME_DIAGNOSTICS.as_ptr().cast(),
        negotiated_capabilities: RUNTIME_NEGOTIATED_CAPABILITIES.as_ptr().cast(),
        behavior: &V2_RUNTIME_BEHAVIOR.0,
    });

#[cfg(feature = "abi_v2_only")]
static V2_MISSING_HOST_REPORT: SyncStatic<NativePluginEntryReportV2> =
    SyncStatic(NativePluginEntryReportV2 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2,
        package_manifest_toml: PLUGIN_MANIFEST.as_ptr().cast(),
        diagnostics: V2_MISSING_HOST_DIAGNOSTICS.as_ptr().cast(),
        negotiated_capabilities: std::ptr::null(),
        behavior: std::ptr::null(),
    });

#[cfg(all(feature = "abi_v2_only", not(feature = "descriptor_export_missing")))]
#[no_mangle]
pub extern "C" fn zircon_native_plugin_descriptor_v2() -> *const NativePluginAbiV2 {
    &V2_DESCRIPTOR.0
}

#[cfg(feature = "abi_v2_only")]
#[no_mangle]
pub unsafe extern "C" fn zircon_native_dynamic_fixture_runtime_entry_v2(
    host_functions: *const NativePluginHostFunctionTableV2,
) -> *const NativePluginEntryReportV2 {
    if host_functions.is_null() {
        return &V2_MISSING_HOST_REPORT.0;
    }
    let host_functions = unsafe { &*host_functions };
    let Some(host_abi_version) = host_functions.host_abi_version else {
        return &V2_MISSING_HOST_REPORT.0;
    };
    if unsafe { host_abi_version() } != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2 {
        return &V2_MISSING_HOST_REPORT.0;
    }
    &V2_RUNTIME_REPORT.0
}

unsafe extern "C" fn fixture_runtime_tick_bridge(
    _call: NativePluginBridgeMethodCallV3,
) -> native::ZrStatus {
    native::ZrStatus::ok()
}

unsafe extern "C" fn fixture_invoke_command(
    command_slot: u32,
    payload: NativePluginByteSliceV2,
    output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    native::catch_native_callback_panic(STATUS_PANIC_DIAGNOSTICS, || unsafe {
        fixture_invoke_command_inner(command_slot, payload, output)
    })
}

unsafe fn fixture_invoke_command_inner(
    command_slot: u32,
    payload: NativePluginByteSliceV2,
    output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    match command_slot {
        COMMAND_ECHO_SLOT => {
            let prefix_status = unsafe { output.write(b"echo:") };
            if prefix_status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
                return prefix_status;
            }
            let payload_status = unsafe { output.write(bytes_from_slice(payload)) };
            if payload_status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
                return payload_status;
            }
            status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_ECHO_DIAGNOSTICS)
        }
        COMMAND_BOUNDED_OVERFLOW_SLOT => {
            let sink_status = unsafe { output.write(b"12345") };
            if sink_status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
                return sink_status;
            }
            status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_ECHO_DIAGNOSTICS)
        }
        COMMAND_ASSET_IMPORT_SLOT => fixture_import_data_json(payload, output),
        COMMAND_PANIC_SLOT => panic!("fixture command panic"),
        _ => status(
            ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
            STATUS_BAD_COMMAND_DIAGNOSTICS,
        ),
    }
}

unsafe fn fixture_import_data_json(
    payload: NativePluginByteSliceV2,
    output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    let Ok((metadata, source_bytes)) = decode_import_request(bytes_from_slice(payload)) else {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_ASSET_IMPORT_INVALID_DIAGNOSTICS,
        );
    };
    if metadata.importer_id != FIXTURE_DATA_IMPORTER_ID {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
            STATUS_DENIED_COMMAND_DIAGNOSTICS,
        );
    }
    let Ok(response) = encode_import_response(&metadata, source_bytes, output.max_output_bytes)
    else {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_ASSET_IMPORT_INVALID_DIAGNOSTICS,
        );
    };
    let sink_status = unsafe { output.write(&response) };
    if sink_status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        return sink_status;
    }
    status(
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        STATUS_ASSET_IMPORT_DIAGNOSTICS,
    )
}

fn decode_import_request(
    payload: &[u8],
) -> Result<(NativeAssetImportRequestMetadata, &[u8]), String> {
    if !payload.starts_with(IMPORT_REQUEST_MAGIC)
        || payload.len() < IMPORT_REQUEST_MAGIC.len() + IMPORT_ENVELOPE_LENGTH_BYTES
    {
        return Err("missing import request magic".to_string());
    }
    let metadata_len_start = IMPORT_REQUEST_MAGIC.len();
    let metadata_len_end = metadata_len_start + IMPORT_ENVELOPE_LENGTH_BYTES;
    let metadata_len = usize::try_from(u64::from_le_bytes(
        payload[metadata_len_start..metadata_len_end]
            .try_into()
            .map_err(|_| "invalid import metadata length".to_string())?,
    ))
    .map_err(|_| "import metadata length exceeds platform limits".to_string())?;
    if metadata_len > MAX_IMPORT_METADATA_BYTES {
        return Err("import metadata exceeds the fixture budget".to_string());
    }
    let metadata_end = metadata_len_end
        .checked_add(metadata_len)
        .ok_or_else(|| "import metadata length overflow".to_string())?;
    if metadata_end > payload.len() {
        return Err("import metadata length exceeds payload".to_string());
    }
    let metadata = serde_json::from_slice(&payload[metadata_len_end..metadata_end])
        .map_err(|error| error.to_string())?;
    let source_bytes = &payload[metadata_end..];
    if source_bytes.len() > MAX_IMPORT_SOURCE_BYTES {
        return Err("import source exceeds the fixture budget".to_string());
    }
    Ok((metadata, source_bytes))
}

fn encode_import_response(
    metadata: &NativeAssetImportRequestMetadata,
    source_bytes: &[u8],
    max_output_bytes: usize,
) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(source_bytes).map_err(|error| error.to_string())?;
    let canonical_json: serde_json::Value =
        serde_json::from_str(text).map_err(|error| error.to_string())?;
    let response_metadata = NativeAssetImportResponseMetadata {
        importer_id: &metadata.importer_id,
        entries: [NativeAssetImportResponseEntry {
            locator: &metadata.source_uri,
            imported_asset: NativeAssetImportResponseAsset {
                data: NativeAssetImportData {
                    uri: &metadata.source_uri,
                    format: "json",
                    text,
                    canonical_json,
                },
            },
            migration_report: NativeAssetImportMigrationReport {
                source_schema_version: 1,
                target_schema_version: 2,
                summary: format!("native fixture migrated {}", metadata.source_path),
            },
            diagnostics: [format!("native fixture imported {}", metadata.source_path)],
        }],
    };
    let mut response = BoundedResponseWriter::new(max_output_bytes, source_bytes.len())?;
    serde_json::to_writer(&mut response, &response_metadata).map_err(|error| error.to_string())?;
    response.finish()
}

unsafe extern "C" fn fixture_save_state(
    output: *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    if output.is_null() {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_BAD_OUTPUT_DIAGNOSTICS,
        );
    }
    *output = owned_bytes(RUNTIME_STATE_BLOB.to_vec());
    status(
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        STATUS_STATE_SAVE_DIAGNOSTICS,
    )
}

unsafe extern "C" fn fixture_restore_state(
    state: NativePluginByteSliceV2,
) -> NativePluginCallbackStatusV2 {
    if bytes_from_slice(state) == RUNTIME_STATE_BLOB {
        status(
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            STATUS_STATE_RESTORE_DIAGNOSTICS,
        )
    } else {
        status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_STATE_RESTORE_INVALID_DIAGNOSTICS,
        )
    }
}

unsafe extern "C" fn fixture_unload() -> NativePluginCallbackStatusV2 {
    status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_UNLOAD_DIAGNOSTICS)
}

unsafe extern "C" fn fixture_stateless_unload() -> NativePluginCallbackStatusV2 {
    status(
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        STATUS_STATELESS_UNLOAD_DIAGNOSTICS,
    )
}

unsafe extern "C" fn fixture_stateless_invoke_command(
    _command_slot: u32,
    _payload: NativePluginByteSliceV2,
    _output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    status(
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
        STATUS_STATELESS_COMMAND_DENIED_DIAGNOSTICS,
    )
}

fn emit_host_v3_runtime_signals(host_functions: *const NativePluginHostFunctionTableV3) {
    if host_functions.is_null() {
        return;
    }
    let host_functions = unsafe { &*host_functions };
    if let Some(host_log) = host_functions.host_log {
        unsafe {
            host_log(
                host_functions,
                2,
                RUNTIME_HOST_LOG_TARGET.as_ptr().cast(),
                RUNTIME_HOST_LOG_MESSAGE.as_ptr().cast(),
            );
        }
    }
    if let Some(host_diagnostic) = host_functions.host_diagnostic {
        unsafe {
            host_diagnostic(
                host_functions,
                RUNTIME_HOST_DIAGNOSTIC_PATH.as_ptr().cast(),
                1.0,
                HOST_DIAGNOSTIC_UNIT.as_ptr().cast(),
                RUNTIME_HOST_DIAGNOSTIC_TAGS.as_ptr().cast(),
            );
        }
    }
}

fn emit_host_v3_editor_signals(host_functions: *const NativePluginHostFunctionTableV3) {
    if host_functions.is_null() {
        return;
    }
    let host_functions = unsafe { &*host_functions };
    if let Some(host_log) = host_functions.host_log {
        unsafe {
            host_log(
                host_functions,
                2,
                EDITOR_HOST_LOG_TARGET.as_ptr().cast(),
                EDITOR_HOST_LOG_MESSAGE.as_ptr().cast(),
            );
        }
    }
    if let Some(host_diagnostic) = host_functions.host_diagnostic {
        unsafe {
            host_diagnostic(
                host_functions,
                EDITOR_HOST_DIAGNOSTIC_PATH.as_ptr().cast(),
                1.0,
                HOST_DIAGNOSTIC_UNIT.as_ptr().cast(),
                EDITOR_HOST_DIAGNOSTIC_TAGS.as_ptr().cast(),
            );
        }
    }
}

#[cfg(test)]
mod tests;
