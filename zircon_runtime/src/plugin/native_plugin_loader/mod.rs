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
    NativePluginAbiV1, NativePluginDescriptor, NativePluginEntryReport, NativePluginEntryReportV1,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL,
};
pub use native_plugin_candidate::NativePluginCandidate;
pub use native_plugin_load_manifest::{NativePluginLoadManifest, NativePluginLoadManifestEntry};
pub use native_plugin_load_report::NativePluginLoadReport;
pub use native_plugin_loader::NativePluginLoader;

const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
