use std::path::Path;
use std::sync::Arc;

use super::{
    NativePluginDiscoveryRefreshTicket, NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
    NativePluginLoadReport, NativePluginLoader,
};

pub fn resolve_native_plugin_discovery_root(root: impl AsRef<Path>) -> NativePluginDiscoveryRoot {
    NativePluginLoader.resolve_discovery_root(root)
}

pub fn request_native_plugin_discovery_refresh(
    root: &NativePluginDiscoveryRoot,
) -> NativePluginDiscoveryRefreshTicket {
    NativePluginLoader.request_discovery_refresh(root)
}

pub fn latest_native_plugin_discovery_snapshot(
    root: &NativePluginDiscoveryRoot,
) -> Option<Arc<NativePluginDiscoverySnapshot>> {
    NativePluginLoader.latest_discovery_snapshot(root)
}

pub fn discover_native_plugins(root: impl AsRef<Path>) -> NativePluginLoadReport {
    NativePluginLoader.discover(root)
}

pub fn refresh_native_plugin_discovery_manifest(
    root: impl AsRef<Path>,
    manifest_path: impl AsRef<Path>,
) -> NativePluginLoadReport {
    NativePluginLoader.refresh_discovery_manifest(root, manifest_path)
}

pub fn remove_discovered_native_plugin_path(
    root: impl AsRef<Path>,
    removed_path: impl AsRef<Path>,
) -> NativePluginLoadReport {
    NativePluginLoader.remove_discovered_path(root, removed_path)
}

pub fn native_plugin_discovery_generation(root: impl AsRef<Path>) -> Option<u64> {
    NativePluginLoader.discovery_generation(root)
}

pub fn discover_native_plugins_from_load_manifest(
    export_root: impl AsRef<Path>,
) -> NativePluginLoadReport {
    NativePluginLoader.discover_from_load_manifest(export_root)
}

pub fn load_discovered_native_plugins(root: impl AsRef<Path>) -> NativePluginLoadReport {
    NativePluginLoader.load_discovered_all(root)
}

pub fn load_discovered_native_runtime_plugins(root: impl AsRef<Path>) -> NativePluginLoadReport {
    NativePluginLoader.load_discovered_runtime(root)
}

pub fn load_discovered_native_editor_plugins(root: impl AsRef<Path>) -> NativePluginLoadReport {
    NativePluginLoader.load_discovered_editor(root)
}

pub fn load_native_plugins_from_load_manifest(
    export_root: impl AsRef<Path>,
) -> NativePluginLoadReport {
    NativePluginLoader.load_all_from_load_manifest(export_root)
}

pub fn load_native_runtime_from_load_manifest(
    export_root: impl AsRef<Path>,
) -> NativePluginLoadReport {
    NativePluginLoader.load_runtime_from_load_manifest(export_root)
}

pub fn load_native_editor_from_load_manifest(
    export_root: impl AsRef<Path>,
) -> NativePluginLoadReport {
    NativePluginLoader.load_editor_from_load_manifest(export_root)
}
