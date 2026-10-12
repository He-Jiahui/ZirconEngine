use std::{cell::Cell, sync::Arc};

use crate::core::framework::bridge::PluginInterface;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ExportTargetPlatform;
use crate::plugin::{
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginModuleKind, PluginModuleManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimePluginBridgeLifecycleState,
    RuntimePluginCatalog, RuntimePluginRegistrationReport,
};

use super::super::super::behavior_calls::NativePluginBehavior;
use super::super::{keys::live_key, NativePluginLiveHost};
use super::{
    lock_loaded_native_plugins, NativePluginLoadReport, NativePluginProjectActivationRequest,
    NativePluginProjectActivationSelection, NativePluginProjectActivationSelectionStatus,
};

trait TransactionBridge: Send + Sync {
    fn value(&self) -> u32;
}

impl PluginInterface for dyn TransactionBridge {
    const INTERFACE_ID: &'static str = "native.plugin.transaction_test.v1";
}

#[derive(Debug)]
struct TransactionBridgeProvider;

impl TransactionBridge for TransactionBridgeProvider {
    fn value(&self) -> u32 {
        7
    }
}

// Match the complete native package admission contract before injecting an
// activation-specific fault. Constructor defaults provide canonical coordinates,
// semver versions and packaging; this fixture supplies explicit runtime coverage.
fn activation_package_manifest(plugin_id: &str, display_name: &str) -> PluginPackageManifest {
    let capability = format!("runtime.plugin.{plugin_id}");
    PluginPackageManifest::new(plugin_id, display_name)
        .with_capability(capability.clone())
        .with_supported_targets([RuntimeTargetMode::ClientRuntime])
        .with_supported_platforms([ExportTargetPlatform::Windows])
        .with_runtime_module(
            PluginModuleManifest::runtime(
                format!("{plugin_id}.runtime"),
                format!("zircon_plugin_{plugin_id}_runtime"),
            )
            .with_capabilities([capability])
            .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        )
}

fn bridge_lifecycle_state() -> RuntimePluginBridgeLifecycleState {
    let manifest = activation_package_manifest("atomic_plugin", "Atomic plugin")
        .with_provided_interface(
            PluginInterfaceManifest::new(<dyn TransactionBridge as PluginInterface>::INTERFACE_ID)
                .with_method(PluginInterfaceMethodManifest::new("value", 0)),
        );
    let mut registration = RuntimePluginRegistrationReport::from_native_package_manifest(manifest);
    let owner = registration
        .extensions
        .intern_plugin_module("atomic_plugin.runtime")
        .expect("test runtime module owner");
    registration
        .extensions
        .export_interface::<dyn TransactionBridge>(owner, Arc::new(TransactionBridgeProvider))
        .expect("test bridge export");
    RuntimePluginBridgeLifecycleState::from_catalog(
        RuntimePluginCatalog::from_registration_reports([registration], []),
    )
}

thread_local! {
    static FIXTURE_UNLOAD_CALLS: Cell<usize> = const { Cell::new(0) };
}

// This stateless fixture exposes no commands. An ABI invocation must reject
// every slot rather than fabricate an accepted command or output payload.
unsafe extern "C" fn reject_fixture_command(
    _slot: u32,
    _payload: super::super::super::abi_declarations::NativePluginByteSliceV3,
    _output: super::super::super::abi_declarations::NativePluginOutputSinkV4,
) -> super::super::super::abi_declarations::NativePluginCallbackStatusV3 {
    super::super::super::abi_declarations::NativePluginCallbackStatusV3 {
        code: super::super::super::abi_declarations::ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
        diagnostics: c"activation fixture declares no commands".as_ptr(),
    }
}

// No state is allocated by this fixture; record that real host cleanup
// reached its ABI callback before acknowledging stateless unload.
unsafe extern "C" fn unload_stateless_fixture(
) -> super::super::super::abi_declarations::NativePluginCallbackStatusV3 {
    FIXTURE_UNLOAD_CALLS.with(|calls| calls.set(calls.get() + 1));
    super::super::super::abi_declarations::NativePluginCallbackStatusV3 {
        code: super::super::super::abi_declarations::ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

fn stateless_plugin_behavior() -> NativePluginBehavior {
    NativePluginBehavior {
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: None,
        event_manifest: None,
        registration_manifest: None,
        command_table: None,
        invoke_command: Some(reject_fixture_command),
        save_state: None,
        restore_state: None,
        unload: Some(unload_stateless_fixture),
    }
}

fn staged_plugin_with_manifest(
    manifest: PluginPackageManifest,
) -> super::super::LoadedNativePlugin {
    let registration =
        RuntimePluginRegistrationReport::from_native_package_manifest(manifest.clone());
    assert!(
        registration.is_success(),
        "activation fixture must pass shared package registration before its target gate: {:?}",
        registration.diagnostics
    );
    let mut plugin = super::super::tests::native_live_host_test_plugin_with_behavior(
        &manifest.id,
        stateless_plugin_behavior(),
    );
    plugin
        .descriptor
        .as_mut()
        .expect("test descriptor")
        .package_manifest = Some(manifest.clone());
    let report = plugin
        .runtime_entry_report
        .as_mut()
        .expect("test runtime entry");
    report.package_manifest = Some(manifest);
    assert!(
        report.behavior_validation.diagnostics.is_empty(),
        "activation fixture must pass shared native behavior validation before target faults: {:?}",
        report.behavior_validation
    );
    let projected = NativePluginLoadReport::from_loaded(vec![plugin]);
    let registrations = projected.runtime_plugin_registration_reports();
    assert_eq!(
        registrations.len(),
        1,
        "fixture must project one runtime package"
    );
    assert!(
        registrations[0].is_success(),
        "activation fixture must pass real projected registration before target faults: {:?}",
        registrations[0].diagnostics
    );
    projected
        .into_loaded()
        .pop()
        .expect("validated fixture must retain its loaded generation")
}

fn staged_plugin_with_registration_failure() -> super::super::LoadedNativePlugin {
    let manifest = activation_package_manifest("atomic_plugin", "Atomic plugin")
        .with_provided_interface(
            PluginInterfaceManifest::new(<dyn TransactionBridge as PluginInterface>::INTERFACE_ID)
                .with_method(PluginInterfaceMethodManifest::new("value", 0)),
        );
    let mut plugin = staged_plugin_with_manifest(manifest);
    let report = plugin
        .runtime_entry_report
        .as_mut()
        .expect("test runtime entry");
    report
        .diagnostics
        .push("runtime registration rejected for regression test".to_string());
    plugin
}

#[test]
fn required_registration_failure_keeps_the_old_generation_and_bridge_table() {
    let host = NativePluginLiveHost::default();
    let old = super::super::tests::native_live_host_test_plugin_with_behavior(
        "atomic_plugin",
        stateless_plugin_behavior(),
    );
    let old_library = Arc::clone(&old.library);
    lock_loaded_native_plugins(&host.loaded)
        .expect("host loaded registry should be available")
        .insert(live_key(PluginModuleKind::Runtime, "atomic_plugin"), old);

    let lifecycle = bridge_lifecycle_state();
    let bridge = lifecycle
        .bridge_table()
        .resolve_weak::<dyn TransactionBridge>();
    assert_eq!(bridge.call(|provider| provider.value()), Ok(7));
    let bridge_before = lifecycle.bridge_table().interface_snapshots();
    let selections = [NativePluginProjectActivationSelection {
        plugin_id: "atomic_plugin".to_string(),
        required: true,
    }];
    let request = NativePluginProjectActivationRequest {
        installed_root: std::path::Path::new("installed"),
        target: super::super::super::NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        selections: &selections,
        bridge_lifecycle: Some(&lifecycle),
    };
    let result = host.activate_runtime_project_plugins_from_report(
        request,
        NativePluginLoadReport::from_loaded(vec![staged_plugin_with_registration_failure()]),
    );

    assert!(!result.committed);
    assert_eq!(result.selections.len(), 1);
    assert_eq!(
        result.selections[0].status,
        NativePluginProjectActivationSelectionStatus::Failed
    );
    assert!(result.selections[0]
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("registration")));
    let loaded =
        lock_loaded_native_plugins(&host.loaded).expect("host loaded registry should be available");
    let retained = loaded
        .get(&live_key(PluginModuleKind::Runtime, "atomic_plugin"))
        .expect("previous generation should remain loaded");
    assert!(Arc::ptr_eq(&old_library, &retained.library));
    assert_eq!(
        bridge_before,
        lifecycle.bridge_table().interface_snapshots()
    );
    assert_eq!(bridge.call(|provider| provider.value()), Ok(7));
}

#[test]
fn required_bridge_gate_failure_keeps_the_old_generation_and_provider() {
    let host = NativePluginLiveHost::default();
    let bridge_manifest = activation_package_manifest("atomic_plugin", "Atomic plugin")
        .with_provided_interface(
            PluginInterfaceManifest::new(<dyn TransactionBridge as PluginInterface>::INTERFACE_ID)
                .with_method(PluginInterfaceMethodManifest::new("value", 0)),
        );
    let old = staged_plugin_with_manifest(bridge_manifest.clone());
    let old_library = Arc::clone(&old.library);
    lock_loaded_native_plugins(&host.loaded)
        .expect("host loaded registry should be available")
        .insert(live_key(PluginModuleKind::Runtime, "atomic_plugin"), old);

    let lifecycle = bridge_lifecycle_state();
    let bridge = lifecycle
        .bridge_table()
        .resolve_weak::<dyn TransactionBridge>();
    assert_eq!(bridge.call(|provider| provider.value()), Ok(7));
    let bridge_before = lifecycle.bridge_table().interface_snapshots();
    let selections = [NativePluginProjectActivationSelection {
        plugin_id: "atomic_plugin".to_string(),
        required: true,
    }];
    let request = NativePluginProjectActivationRequest {
        installed_root: std::path::Path::new("installed"),
        target: super::super::super::NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        selections: &selections,
        // A bridge-bearing package must not be loaded into a host that has no lifecycle owner.
        bridge_lifecycle: None,
    };
    let result = host.activate_runtime_project_plugins_from_report(
        request,
        NativePluginLoadReport::from_loaded(vec![staged_plugin_with_manifest(bridge_manifest)]),
    );

    assert!(!result.committed);
    assert!(result.has_required_failures());
    assert!(
        result.selections[0]
            .diagnostic
            .as_deref()
            .is_some_and(|diagnostic| diagnostic.contains("bridge lifecycle is unavailable")),
        "activation did not reach the missing-lifecycle gate: {result:#?}"
    );
    let loaded =
        lock_loaded_native_plugins(&host.loaded).expect("host loaded registry should be available");
    let retained = loaded
        .get(&live_key(PluginModuleKind::Runtime, "atomic_plugin"))
        .expect("previous generation should remain loaded");
    assert!(Arc::ptr_eq(&old_library, &retained.library));
    assert_eq!(
        bridge_before,
        lifecycle.bridge_table().interface_snapshots()
    );
    assert_eq!(bridge.call(|provider| provider.value()), Ok(7));
    assert!(result.load_report.loaded_plugin_ids.is_empty());
}

#[test]
fn optional_registration_failure_is_reported_without_blocking_required_activation() {
    let host = NativePluginLiveHost::default();
    let optional_manifest = activation_package_manifest("optional_plugin", "Optional plugin");
    let required_manifest = activation_package_manifest("required_plugin", "Required plugin");
    let mut optional = staged_plugin_with_manifest(optional_manifest);
    optional
        .runtime_entry_report
        .as_mut()
        .expect("optional runtime entry")
        .diagnostics
        .push("optional registration rejected for regression test".to_string());
    let required = staged_plugin_with_manifest(required_manifest);
    let selections = [
        NativePluginProjectActivationSelection {
            plugin_id: "optional_plugin".to_string(),
            required: false,
        },
        NativePluginProjectActivationSelection {
            plugin_id: "required_plugin".to_string(),
            required: true,
        },
    ];
    let request = NativePluginProjectActivationRequest {
        installed_root: std::path::Path::new("installed"),
        target: super::super::super::NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        selections: &selections,
        bridge_lifecycle: None,
    };

    let result = host.activate_runtime_project_plugins_from_report(
        request,
        NativePluginLoadReport::from_loaded(vec![optional, required]),
    );

    assert!(
        result.committed,
        "optional failure blocked required activation: {result:#?}"
    );
    assert!(!result.has_required_failures());
    assert_eq!(result.selections.len(), 2);
    let optional_result = result
        .selections
        .iter()
        .find(|selection| selection.plugin_id == "optional_plugin")
        .expect("optional selection result");
    assert_eq!(
        optional_result.status,
        NativePluginProjectActivationSelectionStatus::Failed
    );
    assert!(optional_result
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("optional registration rejected")));
    let required_result = result
        .selections
        .iter()
        .find(|selection| selection.plugin_id == "required_plugin")
        .expect("required selection result");
    assert_eq!(
        required_result.status,
        NativePluginProjectActivationSelectionStatus::Activated
    );
    assert_eq!(
        result.load_report.loaded_plugin_ids,
        vec!["required_plugin".to_string()]
    );
    let loaded =
        lock_loaded_native_plugins(&host.loaded).expect("host loaded registry should be available");
    assert!(loaded
        .get(&live_key(PluginModuleKind::Runtime, "required_plugin"))
        .is_some());
    assert!(loaded
        .get(&live_key(PluginModuleKind::Runtime, "optional_plugin"))
        .is_none());
}

#[test]
fn activation_fixture_passes_projected_admission_and_executes_stateless_cleanup() {
    let plugin = staged_plugin_with_manifest(activation_package_manifest(
        "precondition_plugin",
        "Precondition plugin",
    ));
    let behavior = plugin
        .runtime_entry_report
        .as_ref()
        .expect("fixture runtime entry")
        .behavior
        .as_ref()
        .expect("fixture behavior");
    let rejected = behavior
        .callback_snapshot()
        .invoke_command("undeclared", b"");
    assert_eq!(
        rejected.status_code,
        super::super::super::abi_declarations::ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
    assert!(rejected
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("no v4 command manifest table")));
    let unload_before = FIXTURE_UNLOAD_CALLS.with(Cell::get);
    let cleanup = behavior.unload();
    assert_eq!(
        cleanup.status_code,
        super::super::super::abi_declarations::ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
    assert!(cleanup.diagnostics.is_empty());
    assert_eq!(FIXTURE_UNLOAD_CALLS.with(Cell::get), unload_before + 1);
}
