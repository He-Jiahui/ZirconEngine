use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus};

pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3: u32 = 3;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
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
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV3 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV3,
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

pub const NATIVE_COMMAND_MANIFEST_SCHEMA_V3: &[u8] = b"zircon.native.command-manifest/3\0";
pub const NATIVE_EVENT_MANIFEST_SCHEMA_V3: &[u8] = b"zircon.native.event-manifest/3\0";
pub const NATIVE_EMPTY_CSTR: &[u8] = b"\0";
pub const NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS: &[u8] =
    b"native plugin SDK allocation owner mismatch\0";
const SDK_OWNER_TOKEN_SALT: u64 = 0x5a17_c0de_f11e_d00d;

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

pub unsafe extern "C" fn free_owned_bytes_v2(
    buffer: NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    if buffer.data.is_null() || buffer.capacity == 0 {
        return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR);
    }
    let owner_matches = buffer.owner_token == owner_token(buffer.data, buffer.len, buffer.capacity);
    let _ = unsafe { Vec::from_raw_parts(buffer.data, buffer.len, buffer.capacity) };
    if owner_matches {
        callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR)
    } else {
        callback_status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS,
        )
    }
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
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(callback));
    std::panic::set_hook(previous_hook);
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
        required_capabilities: [$($required_capability:expr),* $(,)?],
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

        static __ZIRCON_NATIVE_SDK_BEHAVIOR_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBehaviorV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginBehaviorV3 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            is_stateless: 1,
            schema_versions: $crate::native::NativePluginSchemaVersionsV3 {
                state_schema_version: 0,
                command_manifest_schema: $crate::native::NATIVE_COMMAND_MANIFEST_SCHEMA_V3
                    .as_ptr()
                    .cast(),
                event_manifest_schema: $crate::native::NATIVE_EVENT_MANIFEST_SCHEMA_V3
                    .as_ptr()
                    .cast(),
            },
            command_manifest: ($command_manifest).as_ptr().cast(),
            event_manifest: ($event_manifest).as_ptr().cast(),
            invoke_command: Some($invoke_command),
            save_state: None,
            restore_state: None,
            unload: None,
        });

        static __ZIRCON_NATIVE_SDK_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($diagnostics).as_ptr().cast(),
            negotiated_capabilities: ($negotiated_capabilities).as_ptr().cast(),
            behavior: __ZIRCON_NATIVE_SDK_BEHAVIOR_V3.as_ptr(),
            bridge_methods: ::core::ptr::null(),
        });

        static __ZIRCON_NATIVE_SDK_MISSING_HOST_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($missing_host_diagnostics).as_ptr().cast(),
            negotiated_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
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
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            package_manifest_toml: b"\0".as_ptr().cast(),
            diagnostics: b"ready\0".as_ptr().cast(),
            negotiated_capabilities: b"runtime.plugin.weather\0".as_ptr().cast(),
            behavior: std::ptr::null(),
            bridge_methods: std::ptr::null(),
        });
    static MISSING: NativePluginStatic<NativePluginEntryReportV3> =
        NativePluginStatic::new(NativePluginEntryReportV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            package_manifest_toml: b"\0".as_ptr().cast(),
            diagnostics: b"missing host\0".as_ptr().cast(),
            negotiated_capabilities: b"\0".as_ptr().cast(),
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
    fn native_panic_guard_maps_panic_to_callback_status() {
        let status = catch_native_callback_panic(b"panic caught\0", || panic!("fixture panic"));

        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC);
    }

    unsafe extern "C" fn host_abi_version() -> u32 {
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    }
}
