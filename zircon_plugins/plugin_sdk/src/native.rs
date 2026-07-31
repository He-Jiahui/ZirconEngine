use std::ffi::{c_char, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::{Deserialize, Serialize};
pub use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus};

#[path = "native_manifest.rs"]
mod native_manifest;

pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3: u32 = 3;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
pub const ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH: u32 = 5;
pub const ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4: u32 = 4;
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3: &[u8] =
    b"zircon_native_plugin_descriptor_v3\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL: &[u8] = ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3;

pub const ZIRCON_NATIVE_PLUGIN_STATUS_OK: u32 = 0;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_ERROR: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_DENIED: u32 = 2;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_PANIC: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginAbiV3 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
    pub requested_capabilities: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginSchemaVersionsV3 {
    pub state_schema_version: u32,
    pub command_manifest_schema: *const c_char,
    pub event_manifest_schema: *const c_char,
    pub registration_manifest_schema: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV3 {
    pub layout_epoch: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub required_capabilities: *const c_char,
    pub denied_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV4,
    pub bridge_methods: *const NativePluginBridgeMethodTableV3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginHostFunctionTableV3 {
    pub abi_version: u32,
    pub host_handle: u64,
    pub granted_capabilities: *const c_char,
    pub host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    pub host_has_capability: Option<NativePluginHostHasCapabilityFnV3>,
    pub host_log: Option<NativePluginHostLogFnV3>,
    pub host_diagnostic: Option<NativePluginHostDiagnosticFnV3>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginByteSliceV2 {
    pub data: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginOwnedByteBufferV2 {
    pub data: *mut u8,
    pub len: usize,
    pub capacity: usize,
    pub owner_token: u64,
    pub free: Option<NativePluginFreeBytesFnV2>,
}

impl NativePluginOwnedByteBufferV2 {
    pub const fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            owner_token: 0,
            free: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginCallbackStatusV2 {
    pub code: u32,
    pub diagnostics: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginOutputSinkV4 {
    pub context: *mut c_void,
    pub max_output_bytes: usize,
    pub write: Option<NativePluginOutputWriteFnV4>,
}

impl NativePluginOutputSinkV4 {
    /// Writes one borrowed chunk into the host-owned command output.
    ///
    /// # Safety
    ///
    /// The sink must be the unmodified value supplied by the host for the active callback. Its
    /// context and writer are valid only for that callback's duration.
    pub unsafe fn write(self, bytes: &[u8]) -> NativePluginCallbackStatusV2 {
        if bytes.len() > self.max_output_bytes {
            return callback_status(
                ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                NATIVE_OUTPUT_SINK_LIMIT_EXCEEDED_DIAGNOSTICS_V4,
            );
        }
        let Some(write) = self.write else {
            return callback_status(
                ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                NATIVE_OUTPUT_SINK_MISSING_WRITER_DIAGNOSTICS_V4,
            );
        };
        unsafe {
            write(
                self.context,
                NativePluginByteSliceV2 {
                    data: bytes.as_ptr(),
                    len: bytes.len(),
                },
            )
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBehaviorV4 {
    pub abi_version: u32,
    pub is_stateless: u32,
    pub schema_versions: NativePluginSchemaVersionsV3,
    pub command_manifest: *const c_char,
    pub event_manifest: *const c_char,
    pub registration_manifest: *const c_char,
    pub invoke_command: Option<NativePluginInvokeCommandFnV4>,
    pub save_state: Option<NativePluginSaveStateFnV3>,
    pub restore_state: Option<NativePluginRestoreStateFnV3>,
    pub unload: Option<NativePluginUnloadFnV3>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBridgeMethodTableV3 {
    pub abi_version: u32,
    pub methods: *const NativePluginBridgeMethodV3,
    pub method_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBridgeMethodV3 {
    pub interface_id: *const c_char,
    pub method_name: *const c_char,
    pub method: Option<NativePluginBridgeMethodFnV3>,
    pub user_data: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBridgeMethodCallV3 {
    pub interface_slot: u32,
    pub method_slot: u32,
    pub payload: ZrByteSlice,
    pub output: ZrByteBufferRef,
    pub user_data: u64,
}

pub type NativePluginFreeBytesFnV2 =
    unsafe extern "C" fn(NativePluginOwnedByteBufferV2) -> NativePluginCallbackStatusV2;
pub type NativePluginOutputWriteFnV4 =
    unsafe extern "C" fn(*mut c_void, NativePluginByteSliceV2) -> NativePluginCallbackStatusV2;
pub type NativePluginInvokeCommandFnV4 = unsafe extern "C" fn(
    u32,
    NativePluginByteSliceV2,
    NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2;
pub type NativePluginSaveStateFnV2 =
    unsafe extern "C" fn(*mut NativePluginOwnedByteBufferV2) -> NativePluginCallbackStatusV2;
pub type NativePluginRestoreStateFnV2 =
    unsafe extern "C" fn(NativePluginByteSliceV2) -> NativePluginCallbackStatusV2;
pub type NativePluginUnloadFnV2 = unsafe extern "C" fn() -> NativePluginCallbackStatusV2;
pub type NativePluginByteSliceV3 = NativePluginByteSliceV2;
pub type NativePluginOwnedByteBufferV3 = NativePluginOwnedByteBufferV2;
pub type NativePluginCallbackStatusV3 = NativePluginCallbackStatusV2;
pub type NativePluginSaveStateFnV3 = NativePluginSaveStateFnV2;
pub type NativePluginRestoreStateFnV3 = NativePluginRestoreStateFnV2;
pub type NativePluginUnloadFnV3 = NativePluginUnloadFnV2;
pub type NativePluginBridgeMethodFnV3 =
    unsafe extern "C" fn(NativePluginBridgeMethodCallV3) -> ZrStatus;
pub type NativePluginHostHasCapabilityFnV3 =
    unsafe extern "C" fn(*const NativePluginHostFunctionTableV3, *const c_char) -> u32;
pub type NativePluginHostLogFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
    u32,
    *const c_char,
    *const c_char,
) -> u32;
pub type NativePluginHostDiagnosticFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
    *const c_char,
    f64,
    *const c_char,
    *const c_char,
) -> u32;

pub const NATIVE_COMMAND_MANIFEST_SCHEMA_V4: &[u8] = b"zircon.native.command-manifest/4\0";
pub const NATIVE_EVENT_MANIFEST_SCHEMA_V3: &[u8] = b"zircon.native.event-manifest/3\0";
pub const NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3: &[u8] =
    b"zircon.native.registration-manifest/3\0";
pub const NATIVE_EMPTY_CSTR: &[u8] = b"\0";
pub const NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS: &[u8] =
    b"native plugin SDK allocation owner mismatch\0";
pub const NATIVE_FREE_INVALID_BUFFER_DIAGNOSTICS: &[u8] =
    b"native plugin SDK buffer descriptor is malformed\0";
pub const NATIVE_OUTPUT_SINK_LIMIT_EXCEEDED_DIAGNOSTICS_V4: &[u8] =
    b"native plugin command output exceeds the host-owned sink limit\0";
pub const NATIVE_OUTPUT_SINK_MISSING_WRITER_DIAGNOSTICS_V4: &[u8] =
    b"native plugin command output sink is missing its host writer\0";
const SDK_OWNER_TOKEN_SALT: u64 = 0x5a17_c0de_f11e_d00d;

pub const NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT: &str = "zircon.native.command-manifest/4";
pub const NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4: usize = 256 * 1024 * 1024;
pub const NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3_TEXT: &str =
    "zircon.native.registration-manifest/3";
pub const NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY_V3_TEXT: &str = "runtime.native.system.worker_safe";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePluginCommandManifestV4 {
    pub schema: String,
    #[serde(default)]
    pub commands: Vec<NativePluginCommandV4>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePluginCommandV4 {
    pub name: String,
    pub slot: u32,
    pub payload_schema: String,
    pub max_output_bytes: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginRegistrationManifestV3 {
    pub schema: String,
    #[serde(default)]
    pub modules: Vec<NativePluginRegistrationModuleV3>,
    #[serde(default)]
    pub systems: Vec<NativePluginRegistrationSystemV3>,
    #[serde(default)]
    pub resources: Vec<NativePluginRegistrationResourceV3>,
    #[serde(default)]
    pub events: Vec<NativePluginRegistrationEventV3>,
    #[serde(default)]
    pub extensions: Vec<NativePluginRegistrationExtensionV3>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginRegistrationModuleV3 {
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginRegistrationSystemV3 {
    pub id: String,
    pub module: String,
    pub stage: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub sets: Vec<String>,
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
    #[serde(default)]
    pub access: Vec<String>,
    #[serde(default)]
    pub thread_affinity: NativePluginRegistrationThreadAffinityV3,
    #[serde(default)]
    pub bridge_interface: Option<String>,
    #[serde(default)]
    pub bridge_method: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativePluginRegistrationThreadAffinityV3 {
    #[default]
    MainThreadOnly,
    WorkerSafe,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginRegistrationResourceV3 {
    pub id: String,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginRegistrationEventV3 {
    pub namespace: String,
    pub name: String,
    #[serde(default)]
    pub stable_hash: u64,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginRegistrationExtensionV3 {
    pub point: String,
    #[serde(default)]
    pub contribution: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
}

pub type NativePluginHostReadyFnV3 = fn(*const NativePluginHostFunctionTableV3);

#[repr(transparent)]
pub struct NativePluginStatic<T>(T);

unsafe impl<T> Sync for NativePluginStatic<T> {}

impl<T> NativePluginStatic<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub const fn get(&self) -> &T {
        &self.0
    }

    pub const fn as_ptr(&self) -> *const T {
        &self.0 as *const T
    }
}

pub struct NativePluginEntryPointV3 {
    report: &'static NativePluginStatic<NativePluginEntryReportV3>,
    missing_host_report: &'static NativePluginStatic<NativePluginEntryReportV3>,
    required_capabilities: &'static [&'static str],
    denied_capabilities: &'static [&'static str],
    on_host_ready: Option<NativePluginHostReadyFnV3>,
}

impl NativePluginEntryPointV3 {
    pub const fn new(
        report: &'static NativePluginStatic<NativePluginEntryReportV3>,
        missing_host_report: &'static NativePluginStatic<NativePluginEntryReportV3>,
        required_capabilities: &'static [&'static str],
        denied_capabilities: &'static [&'static str],
        on_host_ready: Option<NativePluginHostReadyFnV3>,
    ) -> Self {
        Self {
            report,
            missing_host_report,
            required_capabilities,
            denied_capabilities,
            on_host_ready,
        }
    }

    pub fn entry_report(
        &self,
        host_functions: *const NativePluginHostFunctionTableV3,
    ) -> *const NativePluginEntryReportV3 {
        if host_supports_all_capabilities_v3(host_functions, self.required_capabilities)
            && !host_supports_any_capability_v3(host_functions, self.denied_capabilities)
        {
            if let Some(on_host_ready) = self.on_host_ready {
                on_host_ready(host_functions);
            }
            self.report.as_ptr()
        } else {
            self.missing_host_report.as_ptr()
        }
    }
}

pub fn callback_status(code: u32, diagnostics: &'static [u8]) -> NativePluginCallbackStatusV2 {
    NativePluginCallbackStatusV2 {
        code,
        diagnostics: diagnostics.as_ptr().cast(),
    }
}

pub fn owned_bytes(mut bytes: Vec<u8>) -> NativePluginOwnedByteBufferV2 {
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
        free: Some(free_owned_bytes_v2),
    }
}

pub fn command_manifest_v4_to_toml(
    manifest: &NativePluginCommandManifestV4,
) -> Result<String, toml::ser::Error> {
    toml::to_string(manifest)
}

pub fn command_manifest_v4_from_toml(
    text: &str,
) -> Result<NativePluginCommandManifestV4, toml::de::Error> {
    toml::from_str(text)
}

pub fn command_manifest_v4_is_current_and_dense(manifest: &NativePluginCommandManifestV4) -> bool {
    if manifest.schema.trim() != NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT {
        return false;
    }
    let mut names = std::collections::BTreeSet::new();
    manifest
        .commands
        .iter()
        .enumerate()
        .all(|(index, command)| {
            command.slot == u32::try_from(index).unwrap_or(u32::MAX)
                && !command.name.is_empty()
                && !command.payload_schema.trim().is_empty()
                && command.max_output_bytes <= NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4
                && names.insert(command.name.as_str())
        })
}

pub fn registration_manifest_v3_to_toml(
    manifest: &NativePluginRegistrationManifestV3,
) -> Result<String, toml::ser::Error> {
    toml::to_string(manifest)
}

pub fn registration_manifest_v3_from_toml(
    text: &str,
) -> Result<NativePluginRegistrationManifestV3, toml::de::Error> {
    toml::from_str(text)
}

pub fn registration_manifest_v3_schema_is_current(
    manifest: &NativePluginRegistrationManifestV3,
) -> bool {
    manifest.schema.trim() == NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3_TEXT
}

pub unsafe extern "C" fn free_owned_bytes_v2(
    buffer: NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    // The descriptor can cross an FFI boundary. Validate its shape before constructing a Vec.
    if buffer.data.is_null() {
        return if buffer.len == 0 && buffer.capacity == 0 {
            callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR)
        } else {
            callback_status(
                ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                NATIVE_FREE_INVALID_BUFFER_DIAGNOSTICS,
            )
        };
    }
    if buffer.len > buffer.capacity {
        return callback_status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            NATIVE_FREE_INVALID_BUFFER_DIAGNOSTICS,
        );
    }
    if buffer.capacity == 0 {
        return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR);
    }
    if buffer.owner_token != owner_token(buffer.data, buffer.len, buffer.capacity) {
        return callback_status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS,
        );
    }
    let _ = unsafe { Vec::from_raw_parts(buffer.data, buffer.len, buffer.capacity) };
    callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR)
}

pub unsafe fn bytes_from_slice<'a>(slice: NativePluginByteSliceV2) -> &'a [u8] {
    if slice.data.is_null() || slice.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(slice.data, slice.len) }
    }
}

pub fn catch_native_callback_panic<F>(
    panic_diagnostics: &'static [u8],
    callback: F,
) -> NativePluginCallbackStatusV2
where
    F: FnOnce() -> NativePluginCallbackStatusV2,
{
    let result = catch_unwind(AssertUnwindSafe(callback));
    match result {
        Ok(status) => status,
        Err(_) => callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_PANIC, panic_diagnostics),
    }
}

pub fn host_supports_all_capabilities_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capabilities: &[&str],
) -> bool {
    capabilities
        .iter()
        .all(|capability| host_supports_capability_v3(host_functions, capability))
}

pub fn host_supports_any_capability_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capabilities: &[&str],
) -> bool {
    capabilities
        .iter()
        .any(|capability| host_supports_capability_v3(host_functions, capability))
}

pub fn host_supports_capability_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
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
        || host_functions.host_handle == 0
    {
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

pub fn capability_list_contains(capabilities: *const std::ffi::c_char, capability: &str) -> bool {
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

fn owner_token(data: *mut u8, len: usize, capacity: usize) -> u64 {
    SDK_OWNER_TOKEN_SALT ^ data as usize as u64 ^ ((len as u64) << 7) ^ ((capacity as u64) << 17)
}

#[macro_export]
macro_rules! export_native_plugin_descriptor_v3 {
    ($descriptor:expr) => {
        #[no_mangle]
        pub extern "C" fn zircon_native_plugin_descriptor_v3(
        ) -> *const $crate::native::NativePluginAbiV3 {
            ($descriptor).as_ptr()
        }
    };
}

#[macro_export]
macro_rules! export_native_plugin_entry_v3 {
    ($entry_fn:ident, $entry_point:expr) => {
        #[no_mangle]
        pub extern "C" fn $entry_fn(
            host_functions: *const $crate::native::NativePluginHostFunctionTableV3,
        ) -> *const $crate::native::NativePluginEntryReportV3 {
            ($entry_point).entry_report(host_functions)
        }
    };
}

#[macro_export]
macro_rules! native_command_plugin_v3 {
    (
        plugin_id: $plugin_id:expr,
        package_manifest: $package_manifest:expr,
        runtime_entry: $runtime_entry:ident,
        runtime_entry_name: $runtime_entry_name:expr,
        requested_capabilities: $requested_capabilities:expr,
        required_capabilities: [$($required_capability:literal),* $(,)?],
        negotiated_capabilities: $negotiated_capabilities:expr,
        diagnostics: $diagnostics:expr,
        missing_host_diagnostics: $missing_host_diagnostics:expr,
        command_manifest: $command_manifest:expr,
        event_manifest: $event_manifest:expr,
        invoke_command: $invoke_command:path $(,)?
    ) => {
        static __ZIRCON_NATIVE_SDK_DESCRIPTOR_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginAbiV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginAbiV3 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            plugin_id: ($plugin_id).as_ptr().cast(),
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            runtime_entry_name: ($runtime_entry_name).as_ptr().cast(),
            editor_entry_name: ::core::ptr::null(),
            requested_capabilities: ($requested_capabilities).as_ptr().cast(),
        });

        static __ZIRCON_NATIVE_SDK_BEHAVIOR_V4: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBehaviorV4,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginBehaviorV4 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
            is_stateless: 1,
            schema_versions: $crate::native::NativePluginSchemaVersionsV3 {
                state_schema_version: 0,
                command_manifest_schema: $crate::native::NATIVE_COMMAND_MANIFEST_SCHEMA_V4
                    .as_ptr()
                    .cast(),
                event_manifest_schema: $crate::native::NATIVE_EVENT_MANIFEST_SCHEMA_V3
                    .as_ptr()
                    .cast(),
                registration_manifest_schema: ::core::ptr::null(),
            },
            command_manifest: ($command_manifest).as_ptr().cast(),
            event_manifest: ($event_manifest).as_ptr().cast(),
            registration_manifest: ::core::ptr::null(),
            invoke_command: Some($invoke_command),
            save_state: None,
            restore_state: None,
            unload: None,
        });

        const __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_TEXT_V3: &str =
            concat!($($required_capability, "\n",)* "\0");

        static __ZIRCON_NATIVE_SDK_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($diagnostics).as_ptr().cast(),
            negotiated_capabilities: ($negotiated_capabilities).as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            behavior: __ZIRCON_NATIVE_SDK_BEHAVIOR_V4.as_ptr(),
            bridge_methods: ::core::ptr::null(),
        });

        static __ZIRCON_NATIVE_SDK_MISSING_HOST_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($missing_host_diagnostics).as_ptr().cast(),
            negotiated_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            behavior: ::core::ptr::null(),
            bridge_methods: ::core::ptr::null(),
        });

        const __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_V3: &[&str] =
            &[$($required_capability),*];

        static __ZIRCON_NATIVE_SDK_ENTRY_POINT_V3: $crate::native::NativePluginEntryPointV3 =
            $crate::native::NativePluginEntryPointV3::new(
                &__ZIRCON_NATIVE_SDK_REPORT_V3,
                &__ZIRCON_NATIVE_SDK_MISSING_HOST_REPORT_V3,
                __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_V3,
                &[],
                None,
            );

        $crate::export_native_plugin_descriptor_v3!(__ZIRCON_NATIVE_SDK_DESCRIPTOR_V3);
        $crate::export_native_plugin_entry_v3!(
            $runtime_entry,
            __ZIRCON_NATIVE_SDK_ENTRY_POINT_V3
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    static REPORT: NativePluginStatic<NativePluginEntryReportV3> =
        NativePluginStatic::new(NativePluginEntryReportV3 {
            layout_epoch: ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: b"\0".as_ptr().cast(),
            diagnostics: b"ready\0".as_ptr().cast(),
            negotiated_capabilities: b"runtime.plugin.weather\0".as_ptr().cast(),
            required_capabilities: b"runtime.plugin.weather\0".as_ptr().cast(),
            denied_capabilities: b"runtime.plugin.denied\0".as_ptr().cast(),
            behavior: std::ptr::null(),
            bridge_methods: std::ptr::null(),
        });
    static MISSING: NativePluginStatic<NativePluginEntryReportV3> =
        NativePluginStatic::new(NativePluginEntryReportV3 {
            layout_epoch: ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: b"\0".as_ptr().cast(),
            diagnostics: b"missing host\0".as_ptr().cast(),
            negotiated_capabilities: b"\0".as_ptr().cast(),
            required_capabilities: b"runtime.plugin.weather\0".as_ptr().cast(),
            denied_capabilities: b"runtime.plugin.denied\0".as_ptr().cast(),
            behavior: std::ptr::null(),
            bridge_methods: std::ptr::null(),
        });
    static ENTRY: NativePluginEntryPointV3 = NativePluginEntryPointV3::new(
        &REPORT,
        &MISSING,
        &["runtime.plugin.weather"],
        &["runtime.plugin.denied"],
        None,
    );

    #[test]
    fn native_entry_point_selects_report_from_host_capabilities() {
        let granted = b"runtime.plugin.weather\0";
        let host = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            host_handle: 7,
            granted_capabilities: granted.as_ptr().cast(),
            host_abi_version: Some(host_abi_version),
            host_has_capability: None,
            host_log: None,
            host_diagnostic: None,
        };

        assert_eq!(ENTRY.entry_report(&host), REPORT.as_ptr());
    }

    #[test]
    fn native_entry_point_rejects_denied_capability() {
        let granted = b"runtime.plugin.weather\nruntime.plugin.denied\0";
        let host = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            host_handle: 7,
            granted_capabilities: granted.as_ptr().cast(),
            host_abi_version: Some(host_abi_version),
            host_has_capability: None,
            host_log: None,
            host_diagnostic: None,
        };

        assert_eq!(ENTRY.entry_report(&host), MISSING.as_ptr());
    }

    #[test]
    fn native_owned_bytes_round_trips_through_sdk_free_callback() {
        let buffer = owned_bytes(b"payload".to_vec());
        let free = buffer
            .free
            .expect("SDK-owned buffers carry a free callback");
        let status = unsafe { free(buffer) };

        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    }

    #[test]
    fn native_owned_bytes_free_rejects_malformed_length_before_reclaiming() {
        let backing = *b"ok";
        let status = unsafe {
            free_owned_bytes_v2(NativePluginOwnedByteBufferV2 {
                data: backing.as_ptr() as *mut u8,
                len: backing.len(),
                capacity: backing.len() - 1,
                owner_token: 0,
                free: None,
            })
        };

        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    }

    #[test]
    fn native_owned_bytes_free_rejects_null_pointer_with_owned_capacity() {
        let status = unsafe {
            free_owned_bytes_v2(NativePluginOwnedByteBufferV2 {
                data: std::ptr::null_mut(),
                len: 0,
                capacity: 1,
                owner_token: 0,
                free: None,
            })
        };

        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    }

    #[test]
    fn native_owned_bytes_free_rejects_owner_mismatch_without_consuming_allocation() {
        let buffer = owned_bytes(b"payload".to_vec());
        let mut malformed = buffer;
        malformed.owner_token ^= 1;

        let malformed_status = unsafe { free_owned_bytes_v2(malformed) };
        assert_eq!(malformed_status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);

        let recovered_status = unsafe { free_owned_bytes_v2(buffer) };
        assert_eq!(recovered_status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    }

    #[test]
    fn native_panic_guard_maps_panic_to_callback_status() {
        let status = catch_native_callback_panic(b"panic caught\0", || panic!("fixture panic"));

        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC);
    }

    #[test]
    fn native_panic_guard_does_not_replace_the_process_global_hook() {
        let source = include_str!("native.rs");
        let guard = source
            .split("pub fn catch_native_callback_panic")
            .nth(1)
            .and_then(|text| {
                text.split("pub fn host_supports_all_capabilities_v3")
                    .next()
            })
            .expect("read native callback panic guard");

        assert!(guard.contains("catch_unwind(AssertUnwindSafe(callback))"));
        assert!(!guard.contains("std::panic::take_hook()"));
        assert!(!guard.contains("std::panic::set_hook("));
        assert!(!guard.contains("Box::new(|_| {})"));
    }

    #[test]
    fn native_dynamic_registration_manifest_round_trips() {
        let manifest = NativePluginRegistrationManifestV3 {
            schema: NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3_TEXT.to_string(),
            modules: vec![NativePluginRegistrationModuleV3 {
                name: "runtime".to_string(),
                kind: "runtime".to_string(),
            }],
            systems: vec![NativePluginRegistrationSystemV3 {
                id: "fixture.tick".to_string(),
                module: "runtime".to_string(),
                stage: "Update".to_string(),
                order: 10,
                sets: vec!["fixture".to_string()],
                before: vec!["render".to_string()],
                after: vec!["physics".to_string()],
                access: vec!["write:resource:fixture.resource".to_string()],
                thread_affinity: NativePluginRegistrationThreadAffinityV3::WorkerSafe,
                bridge_interface: Some("fixture.runtime".to_string()),
                bridge_method: Some("tick".to_string()),
            }],
            resources: vec![NativePluginRegistrationResourceV3 {
                id: "fixture.resource".to_string(),
                module: Some("runtime".to_string()),
                schema: Some("json".to_string()),
            }],
            events: vec![NativePluginRegistrationEventV3 {
                namespace: "fixture".to_string(),
                name: "echoed".to_string(),
                stable_hash: 42,
                schema: Some("bytes".to_string()),
            }],
            extensions: vec![NativePluginRegistrationExtensionV3 {
                point: "runtime.importer".to_string(),
                contribution: Some("fixture.data_json".to_string()),
                schema: None,
            }],
            capabilities: vec![
                "runtime.plugin.fixture".to_string(),
                NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY_V3_TEXT.to_string(),
            ],
        };

        let text =
            registration_manifest_v3_to_toml(&manifest).expect("registration manifest serializes");
        let parsed =
            registration_manifest_v3_from_toml(&text).expect("registration manifest parses");

        assert!(registration_manifest_v3_schema_is_current(&parsed));
        assert_eq!(parsed, manifest);
    }

    #[test]
    fn native_command_manifest_v4_round_trips_dense_slots_and_output_limits() {
        let manifest = NativePluginCommandManifestV4 {
            schema: NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT.to_string(),
            commands: vec![
                NativePluginCommandV4 {
                    name: "echo\0binary-safe".to_string(),
                    slot: 0,
                    payload_schema: "bytes".to_string(),
                    max_output_bytes: 1024,
                },
                NativePluginCommandV4 {
                    name: "asset.import/data".to_string(),
                    slot: 1,
                    payload_schema: "zircon.asset.import/1".to_string(),
                    max_output_bytes: NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4,
                },
            ],
        };

        let text = command_manifest_v4_to_toml(&manifest).expect("command manifest serializes");
        let parsed =
            command_manifest_v4_from_toml(&text).expect("command manifest parses from TOML");

        assert!(command_manifest_v4_is_current_and_dense(&parsed));
        assert_eq!(parsed, manifest);
    }

    #[test]
    fn native_command_manifest_v4_rejects_whitespace_only_payload_schema() {
        let manifest = NativePluginCommandManifestV4 {
            schema: NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT.to_string(),
            commands: vec![NativePluginCommandV4 {
                name: "echo".to_string(),
                slot: 0,
                payload_schema: " \t\r\n".to_string(),
                max_output_bytes: 1024,
            }],
        };

        assert!(!command_manifest_v4_is_current_and_dense(&manifest));
    }

    #[test]
    fn native_command_manifest_v4_rejects_unknown_root_and_command_fields() {
        for text in [
            r#"schema = "zircon.native.command-manifest/4"
unexpected_root_field = true"#,
            r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "echo"
slot = 0
payload_schema = "bytes"
max_output_bytes = 1
unexpected_command_field = true"#,
        ] {
            assert!(command_manifest_v4_from_toml(text).is_err());
        }
    }

    #[test]
    fn native_output_sink_v4_writes_into_host_owned_context() {
        unsafe extern "C" fn append(
            context: *mut std::ffi::c_void,
            bytes: NativePluginByteSliceV2,
        ) -> NativePluginCallbackStatusV2 {
            let output = unsafe { &mut *context.cast::<Vec<u8>>() };
            output.extend_from_slice(unsafe { bytes_from_slice(bytes) });
            callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR)
        }

        let mut output = Vec::new();
        let sink = NativePluginOutputSinkV4 {
            context: (&mut output as *mut Vec<u8>).cast(),
            max_output_bytes: 8,
            write: Some(append),
        };

        assert_eq!(
            unsafe { sink.write(b"echo:") }.code,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK
        );
        assert_eq!(
            unsafe { sink.write(b"ok") }.code,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK
        );
        assert_eq!(output, b"echo:ok");
    }

    #[test]
    fn native_output_sink_v4_rejects_missing_writer_and_oversized_chunk() {
        let sink = NativePluginOutputSinkV4 {
            context: std::ptr::null_mut(),
            max_output_bytes: 4,
            write: None,
        };

        assert_eq!(
            unsafe { sink.write(b"12345") }.code,
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
        );
        assert_eq!(
            unsafe { sink.write(b"ok") }.code,
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
        );
    }

    unsafe extern "C" fn host_abi_version() -> u32 {
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    }
}
