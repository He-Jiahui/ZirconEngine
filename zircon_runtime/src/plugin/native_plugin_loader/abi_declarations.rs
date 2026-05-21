use std::ffi::c_char;

pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2: u32 = 2;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3: u32 = 3;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V1: &[u8] =
    b"zircon_native_plugin_descriptor_v1\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V2: &[u8] =
    b"zircon_native_plugin_descriptor_v2\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3: &[u8] =
    b"zircon_native_plugin_descriptor_v3\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL: &[u8] = ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3;

pub const ZIRCON_NATIVE_PLUGIN_STATUS_OK: u32 = 0;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_ERROR: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_DENIED: u32 = 2;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_PANIC: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginAbiV1 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginAbiV2 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
    pub requested_capabilities: *const c_char,
}

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
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV1 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV2 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV3 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginHostFunctionTableV2 {
    pub abi_version: u32,
    pub host_handle: u64,
    pub granted_capabilities: *const c_char,
    pub host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    pub host_has_capability:
        Option<unsafe extern "C" fn(*const NativePluginHostFunctionTableV2, *const c_char) -> u32>,
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
    pub(super) fn empty() -> Self {
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBehaviorV3 {
    pub abi_version: u32,
    pub is_stateless: u32,
    pub schema_versions: NativePluginSchemaVersionsV3,
    pub command_manifest: *const c_char,
    pub event_manifest: *const c_char,
    pub invoke_command: Option<NativePluginInvokeCommandFnV3>,
    pub save_state: Option<NativePluginSaveStateFnV3>,
    pub restore_state: Option<NativePluginRestoreStateFnV3>,
    pub unload: Option<NativePluginUnloadFnV3>,
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
pub type NativePluginByteSliceV3 = NativePluginByteSliceV2;
pub type NativePluginOwnedByteBufferV3 = NativePluginOwnedByteBufferV2;
pub type NativePluginCallbackStatusV3 = NativePluginCallbackStatusV2;
pub type NativePluginInvokeCommandFnV3 = NativePluginInvokeCommandFnV2;
pub type NativePluginSaveStateFnV3 = NativePluginSaveStateFnV2;
pub type NativePluginRestoreStateFnV3 = NativePluginRestoreStateFnV2;
pub type NativePluginUnloadFnV3 = NativePluginUnloadFnV2;
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
