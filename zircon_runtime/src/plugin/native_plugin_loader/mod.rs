mod abi_declarations;
mod behavior_calls;
mod behavior_validation;
#[cfg(test)]
pub(super) mod benchmark_harness;
mod bridge_method_abi;
mod bridge_method_bindings;
mod candidate_from_manifest;
mod collect_manifests;
mod compatibility;
mod discover;
mod discover_load_manifest;
mod discovery_refresh;
mod dynamic_library_name;
mod ffi_panic_guard;
mod host_api_adapter;
mod host_callbacks;
mod load_discovered;
mod loaded_native_plugin;
mod native_plugin_abi;
mod native_plugin_candidate;
mod native_plugin_host_handle;
mod native_plugin_live_host;
mod native_plugin_load_manifest;
mod native_plugin_load_report;
mod native_plugin_loader;
mod native_strings;
mod plugin_load_error;
mod registration_manifest;

pub use abi_declarations::{
    NativePluginAbiV3, NativePluginBehaviorV4, NativePluginBridgeMethodCallV3,
    NativePluginBridgeMethodFnV3, NativePluginBridgeMethodTableV3, NativePluginBridgeMethodV3,
    NativePluginByteSliceV2, NativePluginByteSliceV3, NativePluginCallbackStatusV2,
    NativePluginCallbackStatusV3, NativePluginEntryReportV3, NativePluginHostFunctionTableV3,
    NativePluginInvokeCommandFnV4, NativePluginOutputSinkV4, NativePluginOutputWriteFnV4,
    NativePluginOwnedByteBufferV2, NativePluginOwnedByteBufferV3, NativePluginSchemaVersionsV3,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL,
    ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3, ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
pub use behavior_calls::NativePluginBehaviorCallReport;
pub use behavior_validation::{NativePluginBehaviorHealth, NativePluginBehaviorValidationReport};
pub use bridge_method_bindings::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeCall, NativeBridgeMethodBinding,
    NativeBridgeMethodDescriptor, NativeBridgeMethodFn, NativeBridgeMethodManifestError,
};
pub use host_api_adapter::{
    NativeHostApiV3RegistrationScope, NativeHostApiV4RegistrationPolicy,
    NativeHostApiV4RegistrationScope, NativeHostBridgeCallScope,
};
pub use loaded_native_plugin::{LoadedNativePlugin, NativePluginCallbackDiagnostics};
pub use native_plugin_abi::{NativePluginDescriptor, NativePluginEntryReport};
pub use native_plugin_candidate::NativePluginCandidate;
pub use native_plugin_host_handle::{
    discover_native_plugins, discover_native_plugins_from_load_manifest,
    load_discovered_native_editor_plugins, load_discovered_native_plugins,
    load_discovered_native_runtime_plugins, load_native_editor_from_load_manifest,
    load_native_plugins_from_load_manifest, load_native_runtime_from_load_manifest,
    native_plugin_discovery_generation, refresh_native_plugin_discovery_manifest,
    remove_discovered_native_plugin_path, NativePluginHostHandle, NativePluginHostWeakHandle,
};
pub use native_plugin_live_host::{
    NativePluginLiveHost, NativePluginLiveHostBridgeLifecycleReport,
    NativePluginLiveHostBridgeReloadReport, NativePluginLiveHostCommand,
    NativePluginLiveHostDiagnostics, NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome,
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimeDeltaHotUpdateReport,
    NativePluginRuntimeDeltaHotUpdateRequest, NativePluginRuntimeHotUpdateReport,
    NativePluginRuntimePlayModeExitReport, NativePluginRuntimePlayModeSnapshot,
    NativePluginRuntimePluginState, NativePluginRuntimeRegistrationReplayReport,
    NativePluginRuntimeRegistrationSystemReplay, NativePluginRuntimeStateRestoreReport,
    NativePluginRuntimeStateSnapshot, NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND,
    NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
};
pub use native_plugin_load_manifest::{
    NativePluginLoadManifest, NativePluginLoadManifestAbiV3Contract, NativePluginLoadManifestEntry,
};
pub use native_plugin_load_report::{NativePluginLoadProjection, NativePluginLoadReport};
pub use native_plugin_loader::NativePluginLoader;
pub use plugin_load_error::{PluginLoadError, PluginLoadStage};

const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
