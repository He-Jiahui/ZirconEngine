/// Native plugin manifest discovery and load commands.
pub mod discovery;
/// Stable live-host handles for native plugin lifecycle operations.
pub mod host;

pub use super::native_plugin_loader::{
    native_bridge_method_descriptors_from_manifest, LoadedNativePlugin, NativeBridgeCall,
    NativeBridgeMethodBinding, NativeBridgeMethodDescriptor, NativeBridgeMethodFn,
    NativeBridgeMethodManifestError, NativeHostApiV4RegistrationPolicy,
    NativeHostApiV4RegistrationScope, NativeHostBridgeCallScope, NativePluginAbiV3,
    NativePluginBehaviorCallReport, NativePluginBehaviorHealth, NativePluginBehaviorV4,
    NativePluginBehaviorValidationReport, NativePluginBridgeMethodCallV3,
    NativePluginBridgeMethodFnV3, NativePluginBridgeMethodTableV3, NativePluginBridgeMethodV3,
    NativePluginByteSliceV3, NativePluginCallbackDiagnostics, NativePluginCallbackStatusV3,
    NativePluginCandidate, NativePluginDescriptor, NativePluginEditorCommandBinding,
    NativePluginEditorCommandBindingError, NativePluginEntryReport, NativePluginEntryReportV3,
    NativePluginHostFunctionTableV3, NativePluginInvokeCommandFnV4,
    NativePluginLiveHostBridgeLifecycleReport, NativePluginLiveHostBridgeReloadReport,
    NativePluginLiveHostCommand, NativePluginLiveHostDiagnostics, NativePluginLiveHostLoadReport,
    NativePluginLiveHostOutcome, NativePluginLoadManifest, NativePluginLoadManifestAbiV3Contract,
    NativePluginLoadManifestEntry, NativePluginLoadProjection, NativePluginLoadReport,
    NativePluginOutputSinkV4, NativePluginOutputWriteFnV4, NativePluginOwnedByteBufferV3,
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimeDeltaHotUpdateReport,
    NativePluginRuntimeDeltaHotUpdateRequest, NativePluginRuntimeHotUpdateReport,
    NativePluginRuntimePlayModeExitReport, NativePluginRuntimePlayModeSnapshot,
    NativePluginRuntimePluginState, NativePluginRuntimeRegistrationReplayReport,
    NativePluginRuntimeRegistrationSystemReplay, NativePluginRuntimeStateRestoreReport,
    NativePluginRuntimeStateSnapshot, NativePluginSchemaVersionsV3,
    NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL,
    ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3, ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
