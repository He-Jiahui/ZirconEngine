use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F5 native plugin distribution compatibility typed errors",
        &[
            "runtime_15_native_plugin_distribution_compat_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/compatibility.rs",
            "NativeDistributionCompatibilityError::NonNumericVersionComponent",
            "review_f5_native_plugin_distribution_compat_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native plugin registration manifest typed errors",
        &[
            "runtime_15_native_plugin_registration_manifest_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/registration_manifest.rs",
            "NativePluginRegistrationManifestError::UnsupportedSystemStage",
            "review_f5_native_plugin_registration_manifest_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native plugin behavior ABI typed errors",
        &[
            "runtime_15_native_plugin_behavior_abi_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/behavior_calls.rs",
            "NativePluginBehaviorError::UnsupportedAbiVersion",
            "review_f5_native_plugin_behavior_abi_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native bridge method ABI typed errors",
        &[
            "runtime_15_native_bridge_method_abi_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/bridge_method_abi.rs",
            "NativeBridgeMethodAbiError::UnsupportedTableAbiVersion",
            "review_f5_native_bridge_method_abi_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native plugin manifest collection typed errors",
        &[
            "runtime_15_native_plugin_manifest_collection_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/collect_manifests.rs",
            "NativePluginManifestCollectionError::EnumerateRoot",
            "review_f5_native_plugin_manifest_collection_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native plugin manifest candidate typed errors",
        &[
            "runtime_15_native_plugin_manifest_candidate_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/candidate_from_manifest.rs",
            "NativePluginManifestCandidateError::MissingRuntimeOrEditorModule",
            "review_f5_native_plugin_manifest_candidate_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native plugin string helper typed errors",
        &[
            "runtime_15_native_plugin_string_helper_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_strings.rs",
            "NativeStringError::InvalidPackageManifest",
            "review_f5_native_plugin_string_helpers_use_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native plugin descriptor ABI typed errors",
        &[
            "runtime_15_native_plugin_descriptor_abi_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_abi.rs",
            "NativePluginDescriptorAbiError::UnsupportedAbiVersion",
            "review_f5_native_plugin_descriptor_abi_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native plugin entry ABI typed errors",
        &[
            "runtime_15_native_plugin_entry_abi_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_abi.rs",
            "NativePluginEntryAbiError::MissingEntrySymbol",
            "review_f5_native_plugin_entry_abi_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native host API adapter typed errors",
        &[
            "runtime_15_native_host_api_adapter_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/host_api_adapter.rs",
            "NativeHostApiAdapterError::InvalidUtf8",
            "review_f5_native_host_api_adapter_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host loading typed errors",
        &[
            "runtime_15_native_live_host_loading_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/loading.rs",
            "NativePluginLiveHostLoadingError::LiveHostLockPoisoned",
            "review_f5_native_live_host_loading_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host behavior diagnostics typed errors",
        &[
            "runtime_15_native_live_host_behavior_diagnostics_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/diagnostics.rs",
            "NativePluginBehaviorDiagnosticError::FailedStatus",
            "review_f5_native_live_host_behavior_diagnostics_use_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host lifecycle typed errors",
        &[
            "runtime_15_native_live_host_lifecycle_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs",
            "NativePluginLiveHostLifecycleError::HotReloadDidNotLoad",
            "review_f5_native_live_host_lifecycle_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host hot reload typed errors",
        &[
            "runtime_15_native_live_host_hot_reload_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs",
            "NativePluginHotReloadError::StateSchemaMismatch",
            "review_f5_native_live_host_hot_reload_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host registration replay typed errors",
        &[
            "runtime_15_native_live_host_registration_replay_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs",
            "NativePluginRegistrationReplayError::RegisterNativeSystem",
            "review_f5_native_live_host_registration_replay_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host bridge methods typed errors",
        &[
            "runtime_15_native_live_host_bridge_methods_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs",
            "NativePluginBridgeMethodError::MissingDeclaredBridgeMethod",
            "review_f5_native_live_host_bridge_methods_use_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host runtime behavior typed errors",
        &[
            "runtime_15_native_live_host_runtime_behavior_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs",
            "NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded",
            "review_f5_native_live_host_runtime_behavior_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 native live-host bridge lifecycle typed errors",
        &[
            "runtime_15_native_live_host_bridge_lifecycle_typed_errors_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs",
            "NativePluginBridgeLifecycleError::BridgeLifecycleRejected",
            "review_f5_native_live_host_bridge_lifecycle_uses_typed_error",
        ],
    ),
];
