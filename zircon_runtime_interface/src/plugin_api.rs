use crate::buffer::ZrByteSlice;
use crate::handles::ZrRuntimePluginHandle;
use crate::manifest::ZrPluginModuleDescriptorV1;
use crate::runtime_api::ZrHostApiV1;
use crate::status::ZrStatus;

pub const ZR_PLUGIN_ENTRY_SYMBOL_V1: &[u8] = b"zircon_plugin_entry_v1\0";

pub type ZrPluginEntryFnV1 =
    unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrPluginEntryReportV1;
pub type ZrPluginUnloadFnV1 = unsafe extern "C" fn(ZrRuntimePluginHandle) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrPluginApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub unload: Option<ZrPluginUnloadFnV1>,
}

impl ZrPluginApiV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            unload: None,
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
