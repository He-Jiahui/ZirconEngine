use std::ffi::{c_char, CStr};

use serde::Deserialize;
use serde_json::json;
use zircon_plugin_sdk::native::{
    self, bytes_from_slice, callback_status as status, owned_bytes, NativePluginAbiV3,
    NativePluginBehaviorV3, NativePluginByteSliceV2, NativePluginCallbackStatusV2,
    NativePluginEntryPointV3, NativePluginEntryReportV3, NativePluginHostFunctionTableV3,
    NativePluginOwnedByteBufferV2, NativePluginSchemaVersionsV3, NativePluginStatic,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};

#[cfg(feature = "abi_unknown_version")]
const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_ABI_VERSION: u32 = 99;
#[cfg(not(feature = "abi_unknown_version"))]
const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_ABI_VERSION: u32 = ZIRCON_NATIVE_PLUGIN_ABI_VERSION;
const IMPORT_REQUEST_MAGIC: &[u8] = b"ZRIMP001\n";
const IMPORT_RESPONSE_MAGIC: &[u8] = b"ZRIMO001\n";
const FIXTURE_DATA_IMPORTER_ID: &str = "native_dynamic_fixture.data_json";

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");

const PLUGIN_ID: &[u8] = b"native_dynamic_fixture\0";
const RUNTIME_ENTRY: &[u8] = b"zircon_native_dynamic_fixture_runtime_entry_v3\0";
const EDITOR_ENTRY: &[u8] = b"zircon_native_dynamic_fixture_editor_entry_v3\0";
const REQUESTED_CAPABILITIES: &[u8] =
    b"runtime.plugin.native_dynamic_fixture\neditor.extension.native_dynamic_fixture\0";
const RUNTIME_NEGOTIATED_CAPABILITIES: &[u8] = b"runtime.plugin.native_dynamic_fixture\0";
const EDITOR_NEGOTIATED_CAPABILITIES: &[u8] = b"editor.extension.native_dynamic_fixture\0";
const EDITOR_DIAGNOSTICS_V3: &[u8] =
    b"editor entry reached with v3 host ABI table\nnegotiated editor.extension.native_dynamic_fixture\0";
const MISSING_HOST_DIAGNOSTICS_V3: &[u8] = b"native v3 entry missing negotiated host ABI table\0";
const RUNTIME_DIAGNOSTICS_WITH_DENIED_CAPABILITY_V3: &[u8] = b"runtime v3 entry reached with host ABI table\nnegotiated runtime.plugin.native_dynamic_fixture\ndenied capability runtime.plugin.denied_fixture\0";
const RUNTIME_COMMAND_MANIFEST: &[u8] = b"command=echo;payload=bytes\ncommand=mismatched_buffer;payload=bytes\ncommand=panic;payload=bytes\ncommand=asset.import/native_dynamic_fixture.data_json;payload=ZRIMP001\0";
const RUNTIME_EVENT_MANIFEST: &[u8] = b"event=native_dynamic_fixture.echoed;payload=bytes\0";
const RUNTIME_HOST_LOG_TARGET: &[u8] = b"native_dynamic_fixture.runtime\0";
const EDITOR_HOST_LOG_TARGET: &[u8] = b"native_dynamic_fixture.editor\0";
const RUNTIME_HOST_LOG_MESSAGE: &[u8] = b"runtime v3 host log callback reached\0";
const EDITOR_HOST_LOG_MESSAGE: &[u8] = b"editor v3 host log callback reached\0";
const RUNTIME_HOST_DIAGNOSTIC_PATH: &[u8] = b"plugin.native_dynamic_fixture.runtime.entry\0";
const EDITOR_HOST_DIAGNOSTIC_PATH: &[u8] = b"plugin.native_dynamic_fixture.editor.entry\0";
const HOST_DIAGNOSTIC_UNIT: &[u8] = b"count\0";
const RUNTIME_HOST_DIAGNOSTIC_TAGS: &[u8] = b"plugin,native,runtime\0";
const EDITOR_HOST_DIAGNOSTIC_TAGS: &[u8] = b"plugin,native,editor\0";
const EDITOR_COMMAND_MANIFEST: &[u8] = b"\0";
const EDITOR_EVENT_MANIFEST: &[u8] = b"\0";
const STATUS_ECHO_DIAGNOSTICS: &[u8] = b"serialized command echo completed\0";
const STATUS_ASSET_IMPORT_DIAGNOSTICS: &[u8] = b"native fixture asset import completed\0";
const STATUS_ASSET_IMPORT_INVALID_DIAGNOSTICS: &[u8] =
    b"native fixture asset import request was malformed\0";
const STATUS_DENIED_COMMAND_DIAGNOSTICS: &[u8] = b"denied native command unknown\0";
const STATUS_PANIC_DIAGNOSTICS: &[u8] = b"native fixture caught panic during command invocation\0";
const STATUS_BAD_COMMAND_DIAGNOSTICS: &[u8] = b"native command name was null or invalid\0";
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

static DESCRIPTOR_V3: NativePluginStatic<NativePluginAbiV3> =
    NativePluginStatic::new(NativePluginAbiV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_ABI_VERSION,
        plugin_id: PLUGIN_ID.as_ptr().cast(),
        package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
        runtime_entry_name: RUNTIME_ENTRY.as_ptr().cast(),
        editor_entry_name: EDITOR_ENTRY.as_ptr().cast(),
        requested_capabilities: REQUESTED_CAPABILITIES.as_ptr().cast(),
    });

static RUNTIME_BEHAVIOR_V3: NativePluginStatic<NativePluginBehaviorV3> =
    NativePluginStatic::new(NativePluginBehaviorV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        is_stateless: 0,
        schema_versions: NativePluginSchemaVersionsV3 {
            state_schema_version: 3,
            command_manifest_schema: native::NATIVE_COMMAND_MANIFEST_SCHEMA_V3.as_ptr().cast(),
            event_manifest_schema: native::NATIVE_EVENT_MANIFEST_SCHEMA_V3.as_ptr().cast(),
        },
        command_manifest: RUNTIME_COMMAND_MANIFEST.as_ptr().cast(),
        event_manifest: RUNTIME_EVENT_MANIFEST.as_ptr().cast(),
        invoke_command: Some(fixture_invoke_command),
        save_state: Some(fixture_save_state),
        restore_state: Some(fixture_restore_state),
        unload: Some(fixture_unload),
    });

static EDITOR_BEHAVIOR_V3: NativePluginStatic<NativePluginBehaviorV3> =
    NativePluginStatic::new(NativePluginBehaviorV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        is_stateless: 1,
        schema_versions: NativePluginSchemaVersionsV3 {
            state_schema_version: 0,
            command_manifest_schema: std::ptr::null(),
            event_manifest_schema: std::ptr::null(),
        },
        command_manifest: EDITOR_COMMAND_MANIFEST.as_ptr().cast(),
        event_manifest: EDITOR_EVENT_MANIFEST.as_ptr().cast(),
        invoke_command: Some(fixture_stateless_invoke_command),
        save_state: None,
        restore_state: None,
        unload: Some(fixture_stateless_unload),
    });

static RUNTIME_REPORT_V3: NativePluginStatic<NativePluginEntryReportV3> =
    NativePluginStatic::new(NativePluginEntryReportV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
        diagnostics: RUNTIME_DIAGNOSTICS_WITH_DENIED_CAPABILITY_V3
            .as_ptr()
            .cast(),
        negotiated_capabilities: RUNTIME_NEGOTIATED_CAPABILITIES.as_ptr().cast(),
        behavior: RUNTIME_BEHAVIOR_V3.as_ptr(),
        bridge_methods: std::ptr::null(),
    });

static EDITOR_REPORT_V3: NativePluginStatic<NativePluginEntryReportV3> =
    NativePluginStatic::new(NativePluginEntryReportV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
        diagnostics: EDITOR_DIAGNOSTICS_V3.as_ptr().cast(),
        negotiated_capabilities: EDITOR_NEGOTIATED_CAPABILITIES.as_ptr().cast(),
        behavior: EDITOR_BEHAVIOR_V3.as_ptr(),
        bridge_methods: std::ptr::null(),
    });

static MISSING_HOST_REPORT_V3: NativePluginStatic<NativePluginEntryReportV3> =
    NativePluginStatic::new(NativePluginEntryReportV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
        diagnostics: MISSING_HOST_DIAGNOSTICS_V3.as_ptr().cast(),
        negotiated_capabilities: b"\0".as_ptr().cast(),
        behavior: std::ptr::null(),
        bridge_methods: std::ptr::null(),
    });

static RUNTIME_ENTRY_POINT_V3: NativePluginEntryPointV3 = NativePluginEntryPointV3::new(
    &RUNTIME_REPORT_V3,
    &MISSING_HOST_REPORT_V3,
    &["runtime.plugin.native_dynamic_fixture"],
    &["runtime.plugin.denied_fixture"],
    Some(emit_host_v3_runtime_signals),
);

static EDITOR_ENTRY_POINT_V3: NativePluginEntryPointV3 = NativePluginEntryPointV3::new(
    &EDITOR_REPORT_V3,
    &MISSING_HOST_REPORT_V3,
    &["editor.extension.native_dynamic_fixture"],
    &[],
    Some(emit_host_v3_editor_signals),
);

zircon_plugin_sdk::export_native_plugin_descriptor_v3!(DESCRIPTOR_V3);
zircon_plugin_sdk::export_native_plugin_entry_v3!(
    zircon_native_dynamic_fixture_runtime_entry_v3,
    RUNTIME_ENTRY_POINT_V3
);
zircon_plugin_sdk::export_native_plugin_entry_v3!(
    zircon_native_dynamic_fixture_editor_entry_v3,
    EDITOR_ENTRY_POINT_V3
);

unsafe extern "C" fn fixture_invoke_command(
    command_name: *const c_char,
    payload: NativePluginByteSliceV2,
    output: *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    native::catch_native_callback_panic(STATUS_PANIC_DIAGNOSTICS, || unsafe {
        fixture_invoke_command_inner(command_name, payload, output)
    })
}

unsafe fn fixture_invoke_command_inner(
    command_name: *const c_char,
    payload: NativePluginByteSliceV2,
    output: *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    if command_name.is_null() {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_BAD_COMMAND_DIAGNOSTICS,
        );
    }
    if output.is_null() {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_BAD_OUTPUT_DIAGNOSTICS,
        );
    }
    let Ok(command_name) = CStr::from_ptr(command_name).to_str() else {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_BAD_COMMAND_DIAGNOSTICS,
        );
    };
    match command_name {
        "echo" => {
            let bytes = bytes_from_slice(payload);
            let mut response = b"echo:".to_vec();
            response.extend_from_slice(bytes);
            *output = owned_bytes(response);
            status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_ECHO_DIAGNOSTICS)
        }
        "mismatched_buffer" => {
            let mut response = b"mismatch:".to_vec();
            response.extend_from_slice(bytes_from_slice(payload));
            let mut buffer = owned_bytes(response);
            buffer.owner_token ^= 1;
            *output = buffer;
            status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_ECHO_DIAGNOSTICS)
        }
        "asset.import/native_dynamic_fixture.data_json" => {
            fixture_import_data_json(payload, output)
        }
        "panic" => panic!("fixture command panic"),
        _ => status(
            ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
            STATUS_DENIED_COMMAND_DIAGNOSTICS,
        ),
    }
}

unsafe fn fixture_import_data_json(
    payload: NativePluginByteSliceV2,
    output: *mut NativePluginOwnedByteBufferV2,
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
    let Ok(response) = encode_import_response(&metadata, source_bytes) else {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_ASSET_IMPORT_INVALID_DIAGNOSTICS,
        );
    };
    *output = owned_bytes(response);
    status(
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        STATUS_ASSET_IMPORT_DIAGNOSTICS,
    )
}

fn decode_import_request(
    payload: &[u8],
) -> Result<(NativeAssetImportRequestMetadata, &[u8]), String> {
    if !payload.starts_with(IMPORT_REQUEST_MAGIC)
        || payload.len() < IMPORT_REQUEST_MAGIC.len() + std::mem::size_of::<u64>()
    {
        return Err("missing import request magic".to_string());
    }
    let metadata_len_start = IMPORT_REQUEST_MAGIC.len();
    let metadata_len_end = metadata_len_start + std::mem::size_of::<u64>();
    let metadata_len = u64::from_le_bytes(
        payload[metadata_len_start..metadata_len_end]
            .try_into()
            .map_err(|_| "invalid import metadata length".to_string())?,
    ) as usize;
    let metadata_end = metadata_len_end + metadata_len;
    if metadata_end > payload.len() {
        return Err("import metadata length exceeds payload".to_string());
    }
    let metadata = serde_json::from_slice(&payload[metadata_len_end..metadata_end])
        .map_err(|error| error.to_string())?;
    Ok((metadata, &payload[metadata_end..]))
}

fn encode_import_response(
    metadata: &NativeAssetImportRequestMetadata,
    source_bytes: &[u8],
) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(source_bytes).map_err(|error| error.to_string())?;
    let canonical_json: serde_json::Value =
        serde_json::from_str(text).map_err(|error| error.to_string())?;
    let response_metadata = json!({
        "importer_id": metadata.importer_id,
        "entries": [
            {
                "locator": metadata.source_uri,
                "imported_asset": {
                    "Data": {
                        "uri": metadata.source_uri,
                        "format": "json",
                        "text": text,
                        "canonical_json": canonical_json,
                    }
                },
                "migration_report": {
                    "source_schema_version": 1,
                    "target_schema_version": 2,
                    "summary": format!("native fixture migrated {}", metadata.source_path),
                },
                "diagnostics": [
                    format!("native fixture imported {}", metadata.source_path),
                ],
            }
        ],
    });
    let metadata_bytes =
        serde_json::to_vec(&response_metadata).map_err(|error| error.to_string())?;
    let mut response = Vec::with_capacity(
        IMPORT_RESPONSE_MAGIC.len() + std::mem::size_of::<u64>() + metadata_bytes.len(),
    );
    response.extend_from_slice(IMPORT_RESPONSE_MAGIC);
    response.extend_from_slice(&(metadata_bytes.len() as u64).to_le_bytes());
    response.extend_from_slice(&metadata_bytes);
    Ok(response)
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
    _command_name: *const c_char,
    _payload: NativePluginByteSliceV2,
    _output: *mut NativePluginOwnedByteBufferV2,
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
