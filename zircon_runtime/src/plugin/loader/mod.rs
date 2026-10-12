#[path = "abi_declarations.rs"]
mod abi_declarations;
#[path = "behavior_calls.rs"]
mod behavior_calls;
#[path = "behavior_validation.rs"]
mod behavior_validation;
#[cfg(test)]
#[path = "tests/benchmark_harness.rs"]
pub(super) mod benchmark_harness;
#[path = "bridge_method_abi.rs"]
mod bridge_method_abi;
#[path = "bridge_method_bindings.rs"]
mod bridge_method_bindings;
#[path = "candidate_from_manifest.rs"]
mod candidate_from_manifest;
#[path = "collect_manifests.rs"]
mod collect_manifests;
#[path = "compatibility.rs"]
mod compatibility;
#[path = "discover.rs"]
mod discover;
#[path = "discover_load_manifest.rs"]
mod discover_load_manifest;
#[path = "discovery_refresh/mod.rs"]
mod discovery_refresh;
#[path = "dynamic_library_name.rs"]
mod dynamic_library_name;
#[path = "ffi_panic_guard.rs"]
mod ffi_panic_guard;
#[path = "host_api_adapter.rs"]
mod host_api_adapter;
#[path = "host_callbacks.rs"]
mod host_callbacks;
#[path = "load_discovered.rs"]
mod load_discovered;
#[path = "loaded_native_plugin.rs"]
mod loaded_native_plugin;
#[path = "native_artifact_staging.rs"]
mod native_artifact_staging;
#[path = "native_artifact_trust.rs"]
mod native_artifact_trust;
#[path = "native_plugin_abi.rs"]
mod native_plugin_abi;
#[path = "native_plugin_candidate.rs"]
mod native_plugin_candidate;
#[path = "native_plugin_discovery.rs"]
mod native_plugin_discovery;
#[path = "native_plugin_host_handle.rs"]
mod native_plugin_host_handle;
#[path = "live.rs"]
mod native_plugin_live_host;
#[path = "native_plugin_load_manifest.rs"]
mod native_plugin_load_manifest;
#[path = "report.rs"]
mod native_plugin_load_report;
#[path = "loader.rs"]
mod native_plugin_loader;
#[path = "native_strings.rs"]
mod native_strings;
#[path = "receipt.rs"]
mod package_receipt;
#[path = "plugin_load_error.rs"]
mod plugin_load_error;
#[path = "registration_manifest.rs"]
mod registration_manifest;

pub(crate) use candidate_from_manifest::native_library_file_name_for_manifest;

pub use abi_declarations::{
    NativePluginAbiV3, NativePluginBehaviorV4, NativePluginBridgeMethodCallV3,
    NativePluginBridgeMethodFnV3, NativePluginBridgeMethodTableV3, NativePluginBridgeMethodV3,
    NativePluginByteSliceV3, NativePluginCallbackStatusV3, NativePluginEntryReportV3,
    NativePluginHostFunctionTableV3, NativePluginInvokeCommandFnV4, NativePluginOutputSinkV4,
    NativePluginOutputWriteFnV4, NativePluginOwnedByteBufferV3, NativePluginSchemaVersionsV3,
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
pub use discovery_refresh::{
    NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshBudgetKind,
    NativePluginDiscoveryRefreshError, NativePluginDiscoveryRefreshTerminal,
    NativePluginDiscoveryRefreshTicket, NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
};
pub use host_api_adapter::{
    NativeHostApiV4RegistrationPolicy, NativeHostApiV4RegistrationScope, NativeHostBridgeCallScope,
};
pub use loaded_native_plugin::{
    LoadedNativePlugin, NativePluginCallbackDiagnostics, NativePluginEditorCommandBinding,
    NativePluginEditorCommandBindingError,
};
pub use native_artifact_trust::{
    NativePluginArtifactAdmissionError, NativePluginArtifactAdmissionReceipt,
    NativePluginArtifactAuthority, NativePluginArtifactDependency, NativePluginArtifactDigest,
    NativePluginArtifactExpectation, NativePluginArtifactTarget, NativePluginArtifactTrust,
};
pub use native_plugin_abi::{NativePluginDescriptor, NativePluginEntryReport};
pub use native_plugin_candidate::NativePluginCandidate;
pub use native_plugin_discovery::{
    discover_native_plugins, discover_native_plugins_from_load_manifest,
    latest_native_plugin_discovery_snapshot, load_discovered_native_editor_plugins,
    load_discovered_native_editor_plugins_with_authority, load_discovered_native_plugins,
    load_discovered_native_plugins_with_authority, load_discovered_native_runtime_plugins,
    load_discovered_native_runtime_plugins_with_authority, load_native_editor_from_load_manifest,
    load_native_editor_from_load_manifest_with_authority, load_native_plugins_from_load_manifest,
    load_native_plugins_from_load_manifest_with_authority, load_native_runtime_from_load_manifest,
    load_native_runtime_from_load_manifest_with_authority, native_plugin_discovery_generation,
    refresh_native_plugin_discovery_manifest, remove_discovered_native_plugin_path,
    request_native_plugin_discovery_refresh, resolve_native_plugin_discovery_root,
    validate_discovered_native_editor_plugins, validate_discovered_native_runtime_plugins,
    validate_native_editor_from_load_manifest, validate_native_runtime_from_load_manifest,
};
pub use native_plugin_host_handle::{NativePluginHostHandle, NativePluginHostWeakHandle};
pub use native_plugin_live_host::{
    NativePluginLiveHost, NativePluginLiveHostBridgeLifecycleReport,
    NativePluginLiveHostBridgeReloadReport, NativePluginLiveHostCommand,
    NativePluginLiveHostDiagnostics, NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome,
    NativePluginProjectActivationCleanupReceipt, NativePluginProjectActivationRequest,
    NativePluginProjectActivationResult, NativePluginProjectActivationSelection,
    NativePluginProjectActivationSelectionResult, NativePluginProjectActivationSelectionStatus,
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
pub use package_receipt::{
    verify_native_package_receipts, NativePackageDependencyArtifact, NativePackageKeyPolicy,
    NativePackageModuleArtifact, NativePackageReceiptError, NativePackageReceiptPolicy,
    NativePackageReceiptTrust, VerifiedNativePackageProof,
};
pub use plugin_load_error::{PluginLoadError, PluginLoadStage};

const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
