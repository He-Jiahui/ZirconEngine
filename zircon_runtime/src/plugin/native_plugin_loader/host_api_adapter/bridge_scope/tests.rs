use std::sync::{Arc, Barrier, Mutex, OnceLock};
use std::thread;

use zircon_runtime_interface::{ZrByteBufferRef, ZrStatus, ZrStatusCode};

use super::super::super::bridge_method_bindings::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeCall, NativeBridgeMethodBinding,
    NativeBridgeMethodDescriptor, NativeBridgeMethodFn, NativeBridgeMethodManifestError,
};
use super::*;
use crate::core::framework::bridge::PluginInterface;
use crate::plugin::{
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginPackageManifest,
    RuntimeExtensionRegistry,
};

use super::super::context_handles::context_snapshot;

trait NativeWeatherBridge: Send + Sync {}

impl PluginInterface for dyn NativeWeatherBridge {
    const INTERFACE_ID: &'static str = "native.weather.bridge.v1";
}

struct NativeWeatherProvider;

impl NativeWeatherBridge for NativeWeatherProvider {}

#[test]
fn native_bridge_call_context_snapshots_share_the_dispatch_table() {
    let table = RuntimeExtensionRegistry::default().frozen_bridge_table();
    let scope = NativeHostBridgeCallScope::new(table);

    let first = bridge_context_for(scope.handle()).expect("first context snapshot");
    let second = bridge_context_for(scope.handle()).expect("second context snapshot");

    assert!(
        std::ptr::eq(&*first, &*second),
        "bridge calls must pin the same registry context instead of cloning nested ownership"
    );
}

#[test]
fn native_host_bridge_call_scope_clone_keeps_context_until_the_last_owner_drops() {
    let scope = unsafe {
        NativeHostBridgeCallScope::with_methods(
            RuntimeExtensionRegistry::default().frozen_bridge_table(),
            std::iter::empty(),
        )
    };
    let handle = scope.handle();
    let retained = scope.clone();

    assert_eq!(retained.handle(), handle);
    drop(scope);
    assert!(context_snapshot(handle.raw()).is_some());

    drop(retained);
    assert!(context_snapshot(handle.raw()).is_none());
}

#[test]
fn native_host_context_handle_rejects_stale_generation_after_scope_drop() {
    let first =
        NativeHostBridgeCallScope::new(RuntimeExtensionRegistry::default().frozen_bridge_table());
    let stale_handle = first.handle();
    let call = first.api().bridge.call.expect("bridge call callback");
    drop(first);

    let replacement =
        NativeHostBridgeCallScope::new(RuntimeExtensionRegistry::default().frozen_bridge_table());

    let stale_status = unsafe {
        call(
            stale_handle,
            0,
            0,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(stale_status.status_code(), ZrStatusCode::NotFound);
    assert_ne!(stale_handle.raw(), replacement.handle().raw());
}

#[test]
fn native_host_context_scope_drop_blocks_new_calls_while_in_flight_call_finishes() {
    let state = Arc::new(InFlightBridgeCallState {
        entered: Barrier::new(2),
        release: Barrier::new(2),
    });
    *in_flight_bridge_call_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(state.clone());

    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let scope = unsafe {
        NativeHostBridgeCallScope::with_methods(
            table,
            [(
                slot,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_blocking_method),
            )],
        )
    };
    let handle = scope.handle();
    let call = scope.api().bridge.call.expect("bridge call callback");
    let worker = thread::spawn(move || unsafe {
        call(
            handle,
            slot.raw(),
            7,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
        .status_code()
    });

    state.entered.wait();
    drop(scope);
    let stale_status = unsafe {
        call(
            handle,
            slot.raw(),
            7,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };
    state.release.wait();
    let in_flight_status = worker.join().expect("in-flight bridge call worker");

    in_flight_bridge_call_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take();
    assert_eq!(stale_status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(in_flight_status, ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_host_bridge_call_scope_dispatches_registered_method() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let scope = unsafe {
        NativeHostBridgeCallScope::with_methods(
            table.clone(),
            [(
                slot,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        )
    };
    let api = scope.api();
    let payload = b"ping";

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
    assert_eq!(
        table.diagnostics(slot).unwrap().enabled_calls,
        debug_bridge_counter_value(1)
    );
}

#[test]
fn native_host_bridge_context_builds_dense_method_rows_without_tree_probes() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let scope = unsafe {
        NativeHostBridgeCallScope::with_methods(
            table,
            [
                (
                    slot,
                    7,
                    NativeBridgeMethodFn::from_rust(native_bridge_test_method),
                ),
                (
                    slot,
                    4_096,
                    NativeBridgeMethodFn::from_rust(native_bridge_test_method),
                ),
            ],
        )
    };

    let metrics = bridge_context_for(scope.handle())
        .expect("bridge context")
        .method_table_metrics();
    let status = unsafe {
        (scope.api().bridge.call.expect("bridge call callback"))(
            scope.handle(),
            slot.raw(),
            4_096,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(metrics.interface_rows, 1);
    assert_eq!(metrics.method_rows, 1);
    assert_eq!(metrics.occupied_methods, 2);
    assert_eq!(metrics.tree_probes, 0);
    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_host_bridge_dense_dispatch_uses_fixed_width_sparse_slot_directory() {
    let source = include_str!("mod.rs");

    assert!(source.contains("DENSE_BRIDGE_SLOT_DIRECTORY_BITS"));
    assert!(source.contains("DenseBridgeSlotDirectory"));
    assert!(!source.contains("interfaces: Vec<Option<DenseBridgeMethodRow>>"));
    assert!(!source.contains("methods: Vec<Option<NativeBridgeMethodFn>>"));
}

#[test]
fn native_host_bridge_dense_dispatch_keeps_sparse_u32_slots_addressable() {
    let methods = DenseBridgeMethodTable::from_entries([(
        u32::MAX,
        u32::MAX,
        NativeBridgeMethodFn::from_rust(native_bridge_test_method),
    )]);

    assert_eq!(methods.len(), 1);
    assert!(methods.get(u32::MAX, u32::MAX).is_some());
    let metrics = methods.metrics();
    assert_eq!(metrics.interface_rows, 1);
    assert_eq!(metrics.method_rows, 1);
    assert_eq!(metrics.occupied_methods, 1);
    assert_eq!(metrics.tree_probes, 0);
}

#[test]
fn native_host_bridge_dispatch_source_uses_frozen_dense_slots() {
    let source = include_str!("mod.rs");
    let registry_source = include_str!("../context_handles/registry.rs");

    assert!(source.contains("DenseBridgeMethodTable"));
    assert!(source.contains("context.methods.get(interface_slot, method_slot)"));
    assert!(!source.contains("BTreeMap"));
    assert!(registry_source.contains("HOST_CONTEXT_PAGE_SLOTS"));
    assert!(!registry_source.contains("writer.slots.clone"));
    assert!(!registry_source.contains("Vec<Arc<HostContextSlot"));
}

#[test]
fn native_host_bridge_context_pin_owns_only_the_registry_arc() {
    let context_source = include_str!("../context_handles/registry.rs");
    let bridge_source = include_str!("mod.rs");

    assert!(!context_source.contains("BridgeCall(Arc<NativeHostBridgeCallContext>)"));
    assert!(context_source.contains("BridgeCall(NativeHostBridgeCallContext)"));
    assert!(bridge_source.contains("struct NativeHostBridgeCallContextPin"));
    assert!(bridge_source.contains("context: Arc<NativeHostApiV3Context>"));
}

#[test]
fn native_host_bridge_call_scope_builds_method_table_from_interface_metadata() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let scope = unsafe {
        NativeHostBridgeCallScope::from_method_descriptors(
            table.clone(),
            [NativeBridgeMethodDescriptor::new(
                <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        )
    }
    .expect("native bridge method metadata should resolve");
    let api = scope.api();
    let payload = b"ping";

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_host_bridge_call_catches_plugin_method_panic() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let scope = unsafe {
        NativeHostBridgeCallScope::with_methods(
            table.clone(),
            [(
                slot,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_panic_method),
            )],
        )
    };
    let api = scope.api();

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::Panic);
    assert_eq!(
        table.diagnostics(slot).unwrap().enabled_calls,
        debug_bridge_counter_value(1)
    );
}

#[test]
fn native_bridge_method_descriptors_use_package_manifest_metadata() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let manifest = PluginPackageManifest::new("weather", "Weather").with_provided_interface(
        PluginInterfaceManifest::new(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .with_method(PluginInterfaceMethodManifest::new("sample_temperature", 7)),
    );
    let descriptors = native_bridge_method_descriptors_from_manifest(
        &manifest,
        [NativeBridgeMethodBinding::new(
            <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID,
            "sample_temperature",
            NativeBridgeMethodFn::from_rust(native_bridge_test_method),
        )],
    )
    .expect("native bridge descriptors from manifest");
    assert_eq!(descriptors.len(), 1);
    assert_eq!(
        descriptors[0].interface_id(),
        <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID
    );
    assert_eq!(descriptors[0].method_slot(), 7);
    let scope =
        unsafe { NativeHostBridgeCallScope::from_method_descriptors(table, descriptors) }.unwrap();
    let api = scope.api();
    let payload = b"ping";

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            payload.as_ptr(),
            payload.len(),
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::CapabilityDenied);
}

#[test]
fn native_bridge_method_descriptors_reject_missing_manifest_binding() {
    let manifest = PluginPackageManifest::new("weather", "Weather").with_provided_interface(
        PluginInterfaceManifest::new(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
            .with_method(PluginInterfaceMethodManifest::new("sample_temperature", 7)),
    );

    let result = native_bridge_method_descriptors_from_manifest(&manifest, []);

    assert!(matches!(
        result,
        Err(NativeBridgeMethodManifestError::MissingBinding {
            interface_id,
            method_name
        }) if interface_id == <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID
            && method_name == "sample_temperature"
    ));
}

#[test]
fn native_host_bridge_call_scope_rejects_unknown_interface_metadata() {
    let table = RuntimeExtensionRegistry::default().frozen_bridge_table();

    let result = unsafe {
        NativeHostBridgeCallScope::from_method_descriptors(
            table,
            [NativeBridgeMethodDescriptor::new(
                "native.missing.bridge.v1",
                1,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        )
    };

    assert!(matches!(
        result,
        Err(crate::plugin::RuntimeExtensionRegistryError::MissingPluginInterface(
            interface_id
        )) if interface_id == "native.missing.bridge.v1"
    ));
}

#[test]
fn native_host_bridge_call_maps_disabled_provider_to_bridge_not_enabled() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    table.set_enabled(slot, false).unwrap();
    let scope = unsafe {
        NativeHostBridgeCallScope::with_methods(
            table.clone(),
            [(
                slot,
                7,
                NativeBridgeMethodFn::from_rust(native_bridge_test_method),
            )],
        )
    };
    let api = scope.api();

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            7,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::BridgeNotEnabled);
    assert_eq!(
        table.diagnostics(slot).unwrap().not_enabled_calls,
        debug_bridge_counter_value(1)
    );
}

#[test]
fn native_host_bridge_call_reports_absent_slot_and_missing_method() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn NativeWeatherBridge>(owner, Arc::new(NativeWeatherProvider))
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let scope = NativeHostBridgeCallScope::new(table);
    let api = scope.api();

    let absent_slot_status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            99,
            7,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };
    let missing_method_status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            slot.raw(),
            99,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(absent_slot_status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(missing_method_status.status_code(), ZrStatusCode::NotFound);
}

#[test]
fn native_host_bridge_call_checks_entry_status_without_materializing_a_diagnostic_snapshot() {
    let source = include_str!("mod.rs");
    let function = source
        .split_once("unsafe fn native_host_bridge_call_v1_inner")
        .expect("native bridge call implementation should exist")
        .1;

    assert!(!function.contains("interface_snapshot"));
    assert!(function.contains("context.table.entry(slot)"));
}

fn native_bridge_test_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = unsafe {
        call.payload
            .checked_slice(zircon_runtime_interface::ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1)
    }
    .expect("valid native bridge test payload");
    if call.interface_slot == 0 && call.method_slot == 7 && payload == b"ping" {
        status(ZrStatusCode::CapabilityDenied)
    } else {
        status(ZrStatusCode::InvalidArgument)
    }
}

fn native_bridge_panic_method(_call: NativeBridgeCall) -> ZrStatus {
    panic!("native bridge method panic")
}

struct InFlightBridgeCallState {
    entered: Barrier,
    release: Barrier,
}

fn in_flight_bridge_call_state() -> &'static Mutex<Option<Arc<InFlightBridgeCallState>>> {
    static STATE: OnceLock<Mutex<Option<Arc<InFlightBridgeCallState>>>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(None))
}

fn native_bridge_blocking_method(_call: NativeBridgeCall) -> ZrStatus {
    let state = in_flight_bridge_call_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
        .cloned()
        .expect("in-flight bridge call state");
    state.entered.wait();
    state.release.wait();
    status(ZrStatusCode::CapabilityDenied)
}

#[cfg(debug_assertions)]
fn debug_bridge_counter_value(value: u64) -> u64 {
    value
}

#[cfg(not(debug_assertions))]
fn debug_bridge_counter_value(_: u64) -> u64 {
    0
}
