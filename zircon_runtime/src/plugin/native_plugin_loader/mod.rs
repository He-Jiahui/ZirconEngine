mod candidate_from_manifest;
mod collect_manifests;
mod discover;
mod discover_load_manifest;
mod dynamic_library_name;
mod load_discovered;
mod loaded_native_plugin;
mod native_plugin_abi;
mod native_plugin_candidate;
mod native_plugin_load_manifest;
mod native_plugin_load_report;
mod native_plugin_loader;

pub use loaded_native_plugin::LoadedNativePlugin;
pub use native_plugin_abi::{
    NativePluginAbiV1, NativePluginAbiV2, NativePluginBehaviorCallReport, NativePluginBehaviorV2,
    NativePluginByteSliceV2, NativePluginCallbackStatusV2, NativePluginDescriptor,
    NativePluginEntryReport, NativePluginEntryReportV1, NativePluginEntryReportV2,
    NativePluginHostFunctionTableV2, NativePluginOwnedByteBufferV2,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL,
    ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V1, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V2,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
pub use native_plugin_candidate::NativePluginCandidate;
pub use native_plugin_load_manifest::{NativePluginLoadManifest, NativePluginLoadManifestEntry};
pub use native_plugin_load_report::NativePluginLoadReport;
pub use native_plugin_loader::NativePluginLoader;

const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
