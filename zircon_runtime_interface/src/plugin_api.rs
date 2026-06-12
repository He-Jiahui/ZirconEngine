use crate::buffer::{ZrByteBufferRef, ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::ZrRuntimePluginHandle;
use crate::manifest::ZrPluginModuleDescriptorV1;
use crate::plugin_events::ZrPluginEventCallbackFnV1;
use crate::runtime_api::ZrHostApiV1;
use crate::status::ZrStatus;

pub const ZR_PLUGIN_ENTRY_SYMBOL_V1: &[u8] = b"zircon_plugin_entry_v1\0";
pub const ZR_PLUGIN_ENTRY_SYMBOL_V3: &[u8] = b"zircon_plugin_entry_v3\0";

pub type ZrPluginEntryFnV1 =
    unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrPluginEntryReportV1;
pub type ZrPluginEntryFnV3 =
    unsafe extern "C" fn(*const ZrHostApiV3) -> *const ZrPluginEntryReportV1;
pub type ZrPluginUnloadFnV1 = unsafe extern "C" fn(ZrRuntimePluginHandle) -> ZrStatus;
pub type ZrNativeSystemInvokeFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, u64, ZrByteSlice) -> ZrStatus;
pub type ZrHostRegisterSystemFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const ZrSystemRegistrationV1) -> ZrStatus;
pub type ZrHostRegisterComponentFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const ZrComponentDescV1) -> ZrStatus;
pub type ZrHostSpawnCommandFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const u8, usize) -> ZrStatus;
pub type ZrHostAssetRequestFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteSlice, *mut ZrOwnedByteBuffer) -> ZrStatus;
pub type ZrHostEventEmitFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrEventTypeId, *const u8, usize) -> ZrStatus;
pub type ZrHostEventDrainFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrEventTypeId, ZrByteBufferRef) -> ZrStatus;
pub type ZrHostDiagnosticsEmitFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteSlice, ZrByteSlice) -> ZrStatus;
pub type ZrHostDiagnosticsMetricFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteSlice, f64, ZrByteSlice) -> ZrStatus;
pub type ZrPluginSnapshotSaveFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteBufferRef) -> ZrStatus;
pub type ZrPluginSnapshotRestoreFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const u8, usize) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrHostApiV3 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub ecs: ZrHostEcsApiV1,
    pub asset: ZrHostAssetApiV1,
    pub event: ZrHostEventApiV1,
    pub diagnostics: ZrHostDiagnosticsApiV1,
}

impl ZrHostApiV3 {
    pub const fn empty() -> Self {
        Self {
            abi_version: 3,
            size_bytes: core::mem::size_of::<Self>(),
            ecs: ZrHostEcsApiV1::empty(),
            asset: ZrHostAssetApiV1::empty(),
            event: ZrHostEventApiV1::empty(),
            diagnostics: ZrHostDiagnosticsApiV1::empty(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrHostEcsApiV1 {
    pub register_system: Option<ZrHostRegisterSystemFnV1>,
    pub register_component: Option<ZrHostRegisterComponentFnV1>,
    pub spawn_command: Option<ZrHostSpawnCommandFnV1>,
}

impl ZrHostEcsApiV1 {
    pub const fn empty() -> Self {
        Self {
            register_system: None,
            register_component: None,
            spawn_command: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrHostAssetApiV1 {
    pub request: Option<ZrHostAssetRequestFnV1>,
}

impl ZrHostAssetApiV1 {
    pub const fn empty() -> Self {
        Self { request: None }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrHostEventApiV1 {
    pub emit: Option<ZrHostEventEmitFnV1>,
    pub drain: Option<ZrHostEventDrainFnV1>,
}

impl ZrHostEventApiV1 {
    pub const fn empty() -> Self {
        Self {
            emit: None,
            drain: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrHostDiagnosticsApiV1 {
    pub emit: Option<ZrHostDiagnosticsEmitFnV1>,
    pub metric: Option<ZrHostDiagnosticsMetricFnV1>,
}

impl ZrHostDiagnosticsApiV1 {
    pub const fn empty() -> Self {
        Self {
            emit: None,
            metric: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrEventTypeId {
    pub namespace: ZrByteSlice,
    pub name: ZrByteSlice,
    pub stable_hash: u64,
}

impl ZrEventTypeId {
    pub const fn empty() -> Self {
        Self {
            namespace: ZrByteSlice::empty(),
            name: ZrByteSlice::empty(),
            stable_hash: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrSystemRegistrationV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub system_id: ZrByteSlice,
    pub stage: u32,
    pub order: i32,
    pub set_names: *const ZrByteSlice,
    pub set_count: usize,
    pub before: *const ZrByteSlice,
    pub before_count: usize,
    pub after: *const ZrByteSlice,
    pub after_count: usize,
    pub invoke: Option<ZrNativeSystemInvokeFnV1>,
    pub user_data: u64,
}

impl ZrSystemRegistrationV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            system_id: ZrByteSlice::empty(),
            stage: 0,
            order: 0,
            set_names: core::ptr::null(),
            set_count: 0,
            before: core::ptr::null(),
            before_count: 0,
            after: core::ptr::null(),
            after_count: 0,
            invoke: None,
            user_data: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrComponentDescV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub type_id: ZrByteSlice,
    pub display_name: ZrByteSlice,
    pub schema: ZrByteSlice,
    pub storage_kind: u32,
}

impl ZrComponentDescV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            type_id: ZrByteSlice::empty(),
            display_name: ZrByteSlice::empty(),
            schema: ZrByteSlice::empty(),
            storage_kind: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrPluginStateSnapshotApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub save: Option<ZrPluginSnapshotSaveFnV1>,
    pub restore: Option<ZrPluginSnapshotRestoreFnV1>,
}

impl ZrPluginStateSnapshotApiV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            save: None,
            restore: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrPluginApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub unload: Option<ZrPluginUnloadFnV1>,
    pub invoke_event: Option<ZrPluginEventCallbackFnV1>,
}

impl ZrPluginApiV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            unload: None,
            invoke_event: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrPluginEntryReportV1 {
    pub abi_version: u32,
    pub plugin_id: ZrByteSlice,
    pub package_manifest: ZrByteSlice,
    pub modules: *const ZrPluginModuleDescriptorV1,
    pub module_count: usize,
    pub diagnostics: ZrByteSlice,
    pub api: *const ZrPluginApiV1,
}

impl ZrPluginEntryReportV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            plugin_id: ZrByteSlice::empty(),
            package_manifest: ZrByteSlice::empty(),
            modules: core::ptr::null(),
            module_count: 0,
            diagnostics: ZrByteSlice::empty(),
            api: core::ptr::null(),
        }
    }
}
