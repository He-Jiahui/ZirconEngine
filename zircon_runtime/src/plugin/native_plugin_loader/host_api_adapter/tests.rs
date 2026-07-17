use std::marker::PhantomData;
use std::sync::Arc;

use super::super::bridge_method_bindings::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeMethodBinding,
    NativeBridgeMethodManifestError,
};
use super::*;
use crate::core::framework::bridge::PluginInterface;
use crate::plugin::{
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginPackageManifest,
};

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
        Arc::ptr_eq(&first, &second),
        "bridge calls must clone an Arc context instead of deep-cloning the method map"
    );
}

#[test]
fn native_host_api_v3_registers_systems_and_components_into_runtime_registry() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV3RegistrationScope::new(&mut registry, "weather.runtime").unwrap();
    let api = scope.api();
    let set_names = [ZrByteSlice::from_static(b"weather.main")];
    let after = [ZrByteSlice::from_static(b"weather.bootstrap")];
    let system = ZrSystemRegistrationV1 {
        system_id: ZrByteSlice::from_static(b"weather.native_tick"),
        stage: SystemStage::ORDER
            .iter()
            .position(|stage| *stage == SystemStage::Update)
            .unwrap() as u32,
        order: 3,
        set_names: set_names.as_ptr(),
        set_count: set_names.len(),
        after: after.as_ptr(),
        after_count: after.len(),
        ..ZrSystemRegistrationV1::empty(3)
    };
    let component = ZrComponentDescV1 {
        type_id: ZrByteSlice::from_static(b"weather.native_component"),
        display_name: ZrByteSlice::from_static(b"Native Weather Component"),
        schema: ZrByteSlice::from_static(br#"{"fields":[]}"#),
        storage_kind: 1,
        ..ZrComponentDescV1::empty(3)
    };

    let system_status = unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) };
    let component_status =
        unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
    drop(scope);

    assert!(system_status.is_ok());
    assert!(component_status.is_ok());
    let systems = registry.plugin_systems().collect::<Vec<_>>();
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].1.id, "weather.native_tick");
    assert_eq!(systems[0].1.stage, SystemStage::Update);
    assert_eq!(systems[0].1.order, 3);
    assert_eq!(systems[0].1.sets.len(), 1);
    assert_eq!(systems[0].1.constraints.len(), 1);
    let mut world = crate::scene::World::default();
    let native_system = systems[0]
        .1
        .build(&mut world)
        .expect("native ABI system should build into a schedule node");
    assert!(native_system.access().has_conservative_world_access());
    assert!(native_system
        .access()
        .conflicts_with(&crate::scene::ecs::SystemParamAccess::default()));
    assert_eq!(
        native_system
            .access()
            .conflict_kinds_with(&crate::scene::ecs::SystemParamAccess::default()),
        vec![crate::scene::ecs::SystemParamConflictKind::World]
    );
    assert_eq!(registry.components().len(), 1);
    assert_eq!(registry.components()[0].type_id, "weather.native_component");
    assert_eq!(registry.components()[0].plugin_id, "weather");
}

#[test]
fn native_system_enters_schedule_as_conservative_node() {
    native_host_api_v3_registers_systems_and_components_into_runtime_registry();
}

#[test]
fn native_host_api_adapter_reports_unknown_stage_with_typed_error() {
    let stage = SystemStage::ORDER.len() as u32;
    let error = stage_from_abi(stage)
        .expect_err("unknown host API stage should report typed adapter error");

    assert!(matches!(
        &error,
        NativeHostApiAdapterError::UnknownSystemStage { stage: actual } if *actual == stage
    ));
    assert_eq!(
        error.to_string(),
        format!("unknown native system stage {stage}")
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn native_host_api_adapter_utf8_error_preserves_source() {
    let bytes = [0xff];
    let error = unsafe {
        read_utf8(ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        })
    }
    .expect_err("invalid host API byte slice should report typed UTF-8 error");

    assert!(matches!(
        &error,
        NativeHostApiAdapterError::InvalidUtf8 { .. }
    ));
    assert!(
        std::error::Error::source(&error).is_some(),
        "invalid UTF-8 adapter error should preserve Utf8Error source"
    );
}

#[test]
fn native_host_api_v3_rejects_unknown_registration_handles() {
    let api = NativeHostApiV3RegistrationScope {
        handle: ZrRuntimePluginHandle::invalid(),
        _registry: PhantomData,
    }
    .api();
    let system = ZrSystemRegistrationV1 {
        system_id: ZrByteSlice::from_static(b"weather.native_tick"),
        ..ZrSystemRegistrationV1::empty(3)
    };

    let status =
        unsafe { (api.ecs.register_system.unwrap())(ZrRuntimePluginHandle::new(9999), &system) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
}

#[test]
fn native_host_api_v3_exposes_bridge_domain_as_unsupported_until_connected() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV3RegistrationScope::new(&mut registry, "weather.runtime").unwrap();
    let api = scope.api();

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            1,
            2,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
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
    let scope = NativeHostBridgeCallScope::with_methods(
        table.clone(),
        [(
            slot,
            7,
            NativeBridgeMethodFn::from_rust(native_bridge_test_method),
        )],
    );
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
    let scope = NativeHostBridgeCallScope::from_method_descriptors(
        table.clone(),
        [NativeBridgeMethodDescriptor::new(
            <dyn NativeWeatherBridge as PluginInterface>::INTERFACE_ID,
            7,
            NativeBridgeMethodFn::from_rust(native_bridge_test_method),
        )],
    )
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
    let scope = NativeHostBridgeCallScope::with_methods(
        table.clone(),
        [(
            slot,
            7,
            NativeBridgeMethodFn::from_rust(native_bridge_panic_method),
        )],
    );
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
    let scope = NativeHostBridgeCallScope::from_method_descriptors(table, descriptors).unwrap();
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

    let result = NativeHostBridgeCallScope::from_method_descriptors(
        table,
        [NativeBridgeMethodDescriptor::new(
            "native.missing.bridge.v1",
            1,
            NativeBridgeMethodFn::from_rust(native_bridge_test_method),
        )],
    );

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
    let scope = NativeHostBridgeCallScope::with_methods(
        table.clone(),
        [(
            slot,
            7,
            NativeBridgeMethodFn::from_rust(native_bridge_test_method),
        )],
    );
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
fn native_host_api_v3_preserves_dotted_plugin_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV3RegistrationScope::new(&mut registry, "net.rpc.runtime")
        .expect("dotted plugin runtime module owner");
    let api = scope.api();
    let component = ZrComponentDescV1 {
        type_id: ZrByteSlice::from_static(b"net.rpc.NativePayload"),
        display_name: ZrByteSlice::from_static(b"Native RPC Payload"),
        ..ZrComponentDescV1::empty(3)
    };

    let status = unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
    drop(scope);

    assert!(status.is_ok());
    assert_eq!(registry.components().len(), 1);
    assert_eq!(registry.components()[0].plugin_id, "net.rpc");
}

#[test]
fn native_host_bridge_call_checks_entry_status_without_materializing_a_diagnostic_snapshot() {
    let source = include_str!("../host_api_adapter.rs");
    let function = source
        .split_once("unsafe fn native_host_bridge_call_v1_inner")
        .expect("native bridge call implementation should exist")
        .1
        .split_once("unsafe extern \"C\" fn native_host_diagnostics_emit_v1")
        .expect("diagnostics callback should follow native bridge call")
        .0;

    assert!(!function.contains("interface_snapshot"));
    assert!(function.contains("context.table.entry(slot)"));
}

fn native_bridge_test_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = unsafe { call.payload.as_slice() };
    if call.interface_slot == 0 && call.method_slot == 7 && payload == b"ping" {
        status(ZrStatusCode::CapabilityDenied)
    } else {
        status(ZrStatusCode::InvalidArgument)
    }
}

fn native_bridge_panic_method(_call: NativeBridgeCall) -> ZrStatus {
    panic!("native bridge method panic")
}

#[cfg(debug_assertions)]
fn debug_bridge_counter_value(value: u64) -> u64 {
    value
}

#[cfg(not(debug_assertions))]
fn debug_bridge_counter_value(_: u64) -> u64 {
    0
}
