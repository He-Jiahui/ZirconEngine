use std::ffi::c_char;

const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = 1;

const PLUGIN_MANIFEST: &str = concat!(
    r#"id = "native_dynamic_sample"
version = "0.1.0"
display_name = "Native Dynamic Sample"
description = "Real dynamic library sample for ABI v1 native plugin loading."
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_dynamic_sample.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_dynamic_sample_native"
target_modes = ["client_runtime", "server_runtime", "editor_host"]
capabilities = ["runtime.plugin.native_dynamic_sample"]

[[modules]]
name = "native_dynamic_sample.editor"
kind = "editor"
crate_name = "zircon_plugin_native_dynamic_sample_native"
target_modes = ["editor_host"]
capabilities = ["editor.extension.native_dynamic_sample"]
"#,
    "\0"
);

const PLUGIN_ID: &[u8] = b"native_dynamic_sample\0";
const RUNTIME_ENTRY: &[u8] = b"zircon_native_dynamic_sample_runtime_entry_v1\0";
const EDITOR_ENTRY: &[u8] = b"zircon_native_dynamic_sample_editor_entry_v1\0";
const RUNTIME_DIAGNOSTICS: &[u8] = b"runtime entry reached\0";
const EDITOR_DIAGNOSTICS: &[u8] = b"editor entry reached\0";

#[repr(C)]
pub struct NativePluginAbiV1 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
}

#[repr(C)]
pub struct NativePluginEntryReportV1 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
}

struct SyncDescriptor(NativePluginAbiV1);
struct SyncEntryReport(NativePluginEntryReportV1);

unsafe impl Sync for SyncDescriptor {}
unsafe impl Sync for SyncEntryReport {}

static DESCRIPTOR: SyncDescriptor = SyncDescriptor(NativePluginAbiV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    plugin_id: PLUGIN_ID.as_ptr().cast(),
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    runtime_entry_name: RUNTIME_ENTRY.as_ptr().cast(),
    editor_entry_name: EDITOR_ENTRY.as_ptr().cast(),
});

static RUNTIME_REPORT: SyncEntryReport = SyncEntryReport(NativePluginEntryReportV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: RUNTIME_DIAGNOSTICS.as_ptr().cast(),
});

static EDITOR_REPORT: SyncEntryReport = SyncEntryReport(NativePluginEntryReportV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: EDITOR_DIAGNOSTICS.as_ptr().cast(),
});

#[no_mangle]
pub extern "C" fn zircon_native_plugin_descriptor_v1() -> *const NativePluginAbiV1 {
    &DESCRIPTOR.0
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_sample_runtime_entry_v1(
) -> *const NativePluginEntryReportV1 {
    &RUNTIME_REPORT.0
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_sample_editor_entry_v1(
) -> *const NativePluginEntryReportV1 {
    &EDITOR_REPORT.0
}
