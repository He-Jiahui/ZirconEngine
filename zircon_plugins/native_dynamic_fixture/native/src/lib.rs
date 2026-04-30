use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1: u32 = 1;
const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = 2;
const ZIRCON_NATIVE_PLUGIN_STATUS_OK: u32 = 0;
const ZIRCON_NATIVE_PLUGIN_STATUS_ERROR: u32 = 1;
const ZIRCON_NATIVE_PLUGIN_STATUS_DENIED: u32 = 2;
const ZIRCON_NATIVE_PLUGIN_STATUS_PANIC: u32 = 3;
const FIXTURE_HOST_HANDLE_REQUIRED: u64 = 1;
const FIXTURE_OWNER_TOKEN_SALT: u64 = 0x5a17_c0de_f11e_d00d;

const PLUGIN_MANIFEST: &str = concat!(
    r#"id = "native_dynamic_fixture"
version = "0.1.0"
display_name = "Native Dynamic Fixture"
description = "Real dynamic library fixture for ABI v2 native plugin loading."
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_dynamic_fixture.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_dynamic_fixture_native"
target_modes = ["client_runtime", "server_runtime", "editor_host"]
capabilities = ["runtime.plugin.native_dynamic_fixture"]

[[modules]]
name = "native_dynamic_fixture.editor"
kind = "editor"
crate_name = "zircon_plugin_native_dynamic_fixture_native"
target_modes = ["editor_host"]
capabilities = ["editor.extension.native_dynamic_fixture"]
"#,
    "\0"
);

const PLUGIN_ID: &[u8] = b"native_dynamic_fixture\0";
const RUNTIME_ENTRY_V1: &[u8] = b"zircon_native_dynamic_fixture_runtime_entry_v1\0";
const EDITOR_ENTRY_V1: &[u8] = b"zircon_native_dynamic_fixture_editor_entry_v1\0";
const RUNTIME_ENTRY: &[u8] = b"zircon_native_dynamic_fixture_runtime_entry_v2\0";
const EDITOR_ENTRY: &[u8] = b"zircon_native_dynamic_fixture_editor_entry_v2\0";
const REQUESTED_CAPABILITIES: &[u8] =
    b"runtime.plugin.native_dynamic_fixture\neditor.extension.native_dynamic_fixture\0";
const RUNTIME_NEGOTIATED_CAPABILITIES: &[u8] = b"runtime.plugin.native_dynamic_fixture\0";
const EDITOR_NEGOTIATED_CAPABILITIES: &[u8] = b"editor.extension.native_dynamic_fixture\0";
const RUNTIME_DIAGNOSTICS_V1: &[u8] = b"runtime entry reached\0";
const EDITOR_DIAGNOSTICS_V1: &[u8] = b"editor entry reached\0";
const EDITOR_DIAGNOSTICS: &[u8] =
    b"editor entry reached with v2 host ABI table\nnegotiated editor.extension.native_dynamic_fixture\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] = b"native v2 entry missing negotiated host ABI table\0";
const RUNTIME_DIAGNOSTICS_WITH_DENIED_CAPABILITY: &[u8] = b"runtime v2 entry reached with host ABI table\nnegotiated runtime.plugin.native_dynamic_fixture\ndenied capability runtime.plugin.denied_fixture\0";
const RUNTIME_COMMAND_MANIFEST: &[u8] = b"command=echo;payload=bytes\ncommand=mismatched_buffer;payload=bytes\ncommand=panic;payload=bytes\0";
const RUNTIME_EVENT_MANIFEST: &[u8] = b"event=native_dynamic_fixture.echoed;payload=bytes\0";
const EDITOR_COMMAND_MANIFEST: &[u8] = b"\0";
const EDITOR_EVENT_MANIFEST: &[u8] = b"\0";
const STATUS_OK_DIAGNOSTICS: &[u8] = b"\0";
const STATUS_ECHO_DIAGNOSTICS: &[u8] = b"serialized command echo completed\0";
const STATUS_DENIED_COMMAND_DIAGNOSTICS: &[u8] = b"denied native command unknown\0";
const STATUS_PANIC_DIAGNOSTICS: &[u8] = b"native fixture caught panic during command invocation\0";
const STATUS_BAD_COMMAND_DIAGNOSTICS: &[u8] = b"native command name was null or invalid\0";
const STATUS_BAD_OUTPUT_DIAGNOSTICS: &[u8] = b"native command output pointer was null\0";
const STATUS_STATE_SAVE_DIAGNOSTICS: &[u8] = b"state save completed\0";
const STATUS_STATE_RESTORE_DIAGNOSTICS: &[u8] = b"state restore accepted\0";
const STATUS_STATE_RESTORE_INVALID_DIAGNOSTICS: &[u8] = b"state restore rejected invalid blob\0";
const STATUS_UNLOAD_DIAGNOSTICS: &[u8] = b"unload callback reached\0";
const STATUS_STATELESS_UNLOAD_DIAGNOSTICS: &[u8] = b"stateless unload callback reached\0";
const STATUS_FREE_MISMATCH_DIAGNOSTICS: &[u8] = b"allocation/free owner mismatch\0";
const RUNTIME_STATE_BLOB: &[u8] = b"state:v2:native_dynamic_fixture";

#[repr(C)]
pub struct NativePluginAbiV1 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
}

#[repr(C)]
pub struct NativePluginAbiV2 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
    pub requested_capabilities: *const c_char,
}

#[repr(C)]
pub struct NativePluginEntryReportV1 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
}

#[repr(C)]
pub struct NativePluginEntryReportV2 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV2,
}

#[repr(C)]
pub struct NativePluginHostFunctionTableV2 {
    pub abi_version: u32,
    pub host_handle: u64,
    pub granted_capabilities: *const c_char,
    pub host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    pub host_has_capability:
        Option<unsafe extern "C" fn(*const NativePluginHostFunctionTableV2, *const c_char) -> u32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativePluginByteSliceV2 {
    pub data: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativePluginOwnedByteBufferV2 {
    pub data: *mut u8,
    pub len: usize,
    pub capacity: usize,
    pub owner_token: u64,
    pub free: Option<NativePluginFreeBytesFnV2>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativePluginCallbackStatusV2 {
    pub code: u32,
    pub diagnostics: *const c_char,
}

#[repr(C)]
pub struct NativePluginBehaviorV2 {
    pub abi_version: u32,
    pub is_stateless: u32,
    pub command_manifest: *const c_char,
    pub event_manifest: *const c_char,
    pub invoke_command: Option<NativePluginInvokeCommandFnV2>,
    pub save_state: Option<NativePluginSaveStateFnV2>,
    pub restore_state: Option<NativePluginRestoreStateFnV2>,
    pub unload: Option<NativePluginUnloadFnV2>,
}

pub type NativePluginFreeBytesFnV2 =
    unsafe extern "C" fn(NativePluginOwnedByteBufferV2) -> NativePluginCallbackStatusV2;
pub type NativePluginInvokeCommandFnV2 = unsafe extern "C" fn(
    *const c_char,
    NativePluginByteSliceV2,
    *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2;
pub type NativePluginSaveStateFnV2 =
    unsafe extern "C" fn(*mut NativePluginOwnedByteBufferV2) -> NativePluginCallbackStatusV2;
pub type NativePluginRestoreStateFnV2 =
    unsafe extern "C" fn(NativePluginByteSliceV2) -> NativePluginCallbackStatusV2;
pub type NativePluginUnloadFnV2 = unsafe extern "C" fn() -> NativePluginCallbackStatusV2;

struct SyncDescriptorV1(NativePluginAbiV1);
struct SyncDescriptorV2(NativePluginAbiV2);
struct SyncEntryReportV1(NativePluginEntryReportV1);
struct SyncEntryReportV2(NativePluginEntryReportV2);
struct SyncBehaviorV2(NativePluginBehaviorV2);

unsafe impl Sync for SyncDescriptorV1 {}
unsafe impl Sync for SyncDescriptorV2 {}
unsafe impl Sync for SyncEntryReportV1 {}
unsafe impl Sync for SyncEntryReportV2 {}
unsafe impl Sync for SyncBehaviorV2 {}

static DESCRIPTOR_V1: SyncDescriptorV1 = SyncDescriptorV1(NativePluginAbiV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1,
    plugin_id: PLUGIN_ID.as_ptr().cast(),
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    runtime_entry_name: RUNTIME_ENTRY_V1.as_ptr().cast(),
    editor_entry_name: EDITOR_ENTRY_V1.as_ptr().cast(),
});

static DESCRIPTOR: SyncDescriptorV2 = SyncDescriptorV2(NativePluginAbiV2 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    plugin_id: PLUGIN_ID.as_ptr().cast(),
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    runtime_entry_name: RUNTIME_ENTRY.as_ptr().cast(),
    editor_entry_name: EDITOR_ENTRY.as_ptr().cast(),
    requested_capabilities: REQUESTED_CAPABILITIES.as_ptr().cast(),
});

static RUNTIME_REPORT_V1: SyncEntryReportV1 = SyncEntryReportV1(NativePluginEntryReportV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: RUNTIME_DIAGNOSTICS_V1.as_ptr().cast(),
});

static EDITOR_REPORT_V1: SyncEntryReportV1 = SyncEntryReportV1(NativePluginEntryReportV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: EDITOR_DIAGNOSTICS_V1.as_ptr().cast(),
});

static RUNTIME_BEHAVIOR: SyncBehaviorV2 = SyncBehaviorV2(NativePluginBehaviorV2 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    is_stateless: 0,
    command_manifest: RUNTIME_COMMAND_MANIFEST.as_ptr().cast(),
    event_manifest: RUNTIME_EVENT_MANIFEST.as_ptr().cast(),
    invoke_command: Some(fixture_invoke_command),
    save_state: Some(fixture_save_state),
    restore_state: Some(fixture_restore_state),
    unload: Some(fixture_unload),
});

static EDITOR_BEHAVIOR: SyncBehaviorV2 = SyncBehaviorV2(NativePluginBehaviorV2 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    is_stateless: 1,
    command_manifest: EDITOR_COMMAND_MANIFEST.as_ptr().cast(),
    event_manifest: EDITOR_EVENT_MANIFEST.as_ptr().cast(),
    invoke_command: None,
    save_state: None,
    restore_state: None,
    unload: Some(fixture_stateless_unload),
});

static RUNTIME_REPORT: SyncEntryReportV2 = SyncEntryReportV2(NativePluginEntryReportV2 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: RUNTIME_DIAGNOSTICS_WITH_DENIED_CAPABILITY.as_ptr().cast(),
    negotiated_capabilities: RUNTIME_NEGOTIATED_CAPABILITIES.as_ptr().cast(),
    behavior: &RUNTIME_BEHAVIOR.0,
});

static EDITOR_REPORT: SyncEntryReportV2 = SyncEntryReportV2(NativePluginEntryReportV2 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: EDITOR_DIAGNOSTICS.as_ptr().cast(),
    negotiated_capabilities: EDITOR_NEGOTIATED_CAPABILITIES.as_ptr().cast(),
    behavior: &EDITOR_BEHAVIOR.0,
});

static MISSING_HOST_REPORT: SyncEntryReportV2 = SyncEntryReportV2(NativePluginEntryReportV2 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: MISSING_HOST_DIAGNOSTICS.as_ptr().cast(),
    negotiated_capabilities: b"\0".as_ptr().cast(),
    behavior: std::ptr::null(),
});

#[no_mangle]
pub extern "C" fn zircon_native_plugin_descriptor_v1() -> *const NativePluginAbiV1 {
    &DESCRIPTOR_V1.0
}

#[no_mangle]
pub extern "C" fn zircon_native_plugin_descriptor_v2() -> *const NativePluginAbiV2 {
    &DESCRIPTOR.0
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_fixture_runtime_entry_v1(
) -> *const NativePluginEntryReportV1 {
    &RUNTIME_REPORT_V1.0
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_fixture_editor_entry_v1() -> *const NativePluginEntryReportV1
{
    &EDITOR_REPORT_V1.0
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_fixture_runtime_entry_v2(
    host_functions: *const NativePluginHostFunctionTableV2,
) -> *const NativePluginEntryReportV2 {
    if host_supports_capability(host_functions, "runtime.plugin.native_dynamic_fixture")
        && !host_supports_capability(host_functions, "runtime.plugin.denied_fixture")
    {
        &RUNTIME_REPORT.0
    } else {
        &MISSING_HOST_REPORT.0
    }
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_fixture_editor_entry_v2(
    host_functions: *const NativePluginHostFunctionTableV2,
) -> *const NativePluginEntryReportV2 {
    if host_supports_capability(host_functions, "editor.extension.native_dynamic_fixture") {
        &EDITOR_REPORT.0
    } else {
        &MISSING_HOST_REPORT.0
    }
}

unsafe extern "C" fn fixture_invoke_command(
    command_name: *const c_char,
    payload: NativePluginByteSliceV2,
    output: *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        fixture_invoke_command_inner(command_name, payload, output)
    }));
    std::panic::set_hook(previous_hook);
    match result {
        Ok(status) => status,
        Err(_) => status(ZIRCON_NATIVE_PLUGIN_STATUS_PANIC, STATUS_PANIC_DIAGNOSTICS),
    }
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
        "panic" => panic!("fixture command panic"),
        _ => status(
            ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
            STATUS_DENIED_COMMAND_DIAGNOSTICS,
        ),
    }
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

unsafe extern "C" fn fixture_free_bytes(
    buffer: NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    if buffer.data.is_null() || buffer.capacity == 0 {
        return status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_OK_DIAGNOSTICS);
    }
    let owner_matches = buffer.owner_token == owner_token(buffer.data, buffer.len, buffer.capacity);
    let _ = Vec::from_raw_parts(buffer.data, buffer.len, buffer.capacity);
    if owner_matches {
        status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_OK_DIAGNOSTICS)
    } else {
        status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATUS_FREE_MISMATCH_DIAGNOSTICS,
        )
    }
}

fn status(code: u32, diagnostics: &[u8]) -> NativePluginCallbackStatusV2 {
    NativePluginCallbackStatusV2 {
        code,
        diagnostics: diagnostics.as_ptr().cast(),
    }
}

fn owned_bytes(mut bytes: Vec<u8>) -> NativePluginOwnedByteBufferV2 {
    let data = bytes.as_mut_ptr();
    let len = bytes.len();
    let capacity = bytes.capacity();
    let owner_token = owner_token(data, len, capacity);
    std::mem::forget(bytes);
    NativePluginOwnedByteBufferV2 {
        data,
        len,
        capacity,
        owner_token,
        free: Some(fixture_free_bytes),
    }
}

fn owner_token(data: *mut u8, len: usize, capacity: usize) -> u64 {
    FIXTURE_OWNER_TOKEN_SALT
        ^ data as usize as u64
        ^ ((len as u64) << 7)
        ^ ((capacity as u64) << 17)
}

unsafe fn bytes_from_slice<'a>(slice: NativePluginByteSliceV2) -> &'a [u8] {
    if slice.data.is_null() || slice.len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(slice.data, slice.len)
    }
}

fn host_supports_capability(
    host_functions: *const NativePluginHostFunctionTableV2,
    capability: &str,
) -> bool {
    if host_functions.is_null() {
        return false;
    }
    let host_functions = unsafe { &*host_functions };
    let host_version = host_functions
        .host_abi_version
        .map(|host_abi_version| unsafe { host_abi_version() })
        .unwrap_or_default();
    if host_functions.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION
        || host_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    {
        return false;
    }
    if host_functions.host_handle != FIXTURE_HOST_HANDLE_REQUIRED {
        return false;
    }
    if let Some(host_has_capability) = host_functions.host_has_capability {
        let Ok(capability) = CString::new(capability) else {
            return false;
        };
        return unsafe { host_has_capability(host_functions, capability.as_ptr()) }
            == ZIRCON_NATIVE_PLUGIN_STATUS_OK;
    }
    capability_list_contains(host_functions.granted_capabilities, capability)
}

fn capability_list_contains(capabilities: *const c_char, capability: &str) -> bool {
    if capabilities.is_null() {
        return false;
    }
    let Ok(capabilities) = unsafe { CStr::from_ptr(capabilities) }.to_str() else {
        return false;
    };
    capabilities
        .split(|character| matches!(character, '\n' | ',' | ';'))
        .map(str::trim)
        .any(|entry| entry == capability)
}
