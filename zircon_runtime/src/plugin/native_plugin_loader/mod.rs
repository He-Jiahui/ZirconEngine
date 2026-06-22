mod abi_declarations;
mod behavior_calls;
mod behavior_validation;
mod bridge_method_abi;
mod bridge_method_bindings;
mod candidate_from_manifest;
mod collect_manifests;
mod discover;
mod discover_load_manifest;
mod dynamic_library_name;
mod ffi_panic_guard;
mod host_api_adapter;
mod host_callbacks;
mod load_discovered;
mod loaded_native_plugin;
mod native_plugin_abi;
mod native_plugin_candidate;
mod native_plugin_live_host;
mod native_plugin_load_manifest;
mod native_plugin_load_report;
mod native_plugin_loader;
mod native_strings;

pub use abi_declarations::{
    NativePluginAbiV3, NativePluginBehaviorV3, NativePluginBridgeMethodCallV3,
    NativePluginBridgeMethodFnV3, NativePluginBridgeMethodTableV3, NativePluginBridgeMethodV3,
    NativePluginByteSliceV2, NativePluginByteSliceV3, NativePluginCallbackStatusV2,
    NativePluginCallbackStatusV3, NativePluginEntryReportV3, NativePluginHostFunctionTableV3,
    NativePluginOwnedByteBufferV2, NativePluginOwnedByteBufferV3, NativePluginSchemaVersionsV3,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
pub use behavior_calls::NativePluginBehaviorCallReport;
pub use behavior_validation::{NativePluginBehaviorHealth, NativePluginBehaviorValidationReport};
pub use bridge_method_bindings::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeCall, NativeBridgeMethodBinding,
    NativeBridgeMethodDescriptor, NativeBridgeMethodFn, NativeBridgeMethodManifestError,
};
pub use host_api_adapter::{NativeHostApiV3RegistrationScope, NativeHostBridgeCallScope};
pub use loaded_native_plugin::LoadedNativePlugin;
pub use native_plugin_abi::{NativePluginDescriptor, NativePluginEntryReport};
pub use native_plugin_candidate::NativePluginCandidate;
pub use native_plugin_live_host::{
    NativePluginLiveHost, NativePluginLiveHostBridgeLifecycleReport,
    NativePluginLiveHostBridgeReloadReport, NativePluginLiveHostCommand,
    NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome, NativePluginRuntimeBehaviorCall,
    NativePluginRuntimeBehaviorDescriptor, NativePluginRuntimeCommandDispatchReport,
    NativePluginRuntimeHotUpdateReport, NativePluginRuntimePlayModeExitReport,
    NativePluginRuntimePlayModeSnapshot, NativePluginRuntimePluginState,
    NativePluginRuntimeStateRestoreReport, NativePluginRuntimeStateSnapshot,
    NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
};
pub use native_plugin_load_manifest::{
    NativePluginLoadManifest, NativePluginLoadManifestAbiV3Contract, NativePluginLoadManifestEntry,
};
pub use native_plugin_load_report::NativePluginLoadReport;
pub use native_plugin_loader::NativePluginLoader;

const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
