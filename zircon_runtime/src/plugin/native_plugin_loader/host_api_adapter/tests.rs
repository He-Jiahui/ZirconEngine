use std::marker::PhantomData;
use std::sync::{Arc, Barrier, Mutex, OnceLock};
use std::thread;

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
    assert_eq!(
        native_system.thread_affinity(),
        crate::scene::ecs::SceneSystemThreadAffinity::MainThreadOnly
    );
    assert!(!native_system.supports_worldless_execution());
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
fn native_host_api_v4_registers_authorized_worker_safe_typed_access() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::new(
            ["runtime.native.system.worker_safe"],
            ["weather.solver"],
        ),
    )
    .unwrap();
    let api = scope.api();
    let component = ZrComponentDescV1 {
        type_id: ZrByteSlice::from_static(b"weather.native_component"),
        display_name: ZrByteSlice::from_static(b"Native Weather Component"),
        schema: ZrByteSlice::from_static(br#"{"fields":[]}"#),
        storage_kind: 1,
        ..ZrComponentDescV1::empty(4)
    };
    let component_status =
        unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
    assert!(component_status.is_ok());

    let accesses = [
        ZrNativeSystemAccessV1 {
            abi_version: 1,
            size_bytes: core::mem::size_of::<ZrNativeSystemAccessV1>(),
            mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
            domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
            stable_id: ZrByteSlice::from_static(b"weather.native_component"),
        },
        ZrNativeSystemAccessV1 {
            abi_version: 1,
            size_bytes: core::mem::size_of::<ZrNativeSystemAccessV1>(),
            mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1,
            domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1,
            stable_id: ZrByteSlice::from_static(b"weather.solver"),
        },
    ];
    let system = ZrSystemRegistrationV2 {
        system_id: ZrByteSlice::from_static(b"weather.native_tick_v4"),
        stage: SystemStage::ORDER
            .iter()
            .position(|stage| *stage == SystemStage::Update)
            .unwrap() as u32,
        accesses: accesses.as_ptr(),
        access_count: accesses.len(),
        thread_affinity: ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1,
        ..ZrSystemRegistrationV2::empty(4)
    };
    let system_status = unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) };
    assert!(system_status.is_ok());
    drop(scope);

    let mut world = crate::scene::World::default();
    world
        .register_component_type(ComponentTypeDescriptor::new(
            "weather.native_component",
            "weather",
            "Native Weather Component",
        ))
        .unwrap();
    let native_system = registry
        .plugin_systems()
        .next()
        .expect("V4 system should be registered")
        .1
        .build(&mut world)
        .expect("V4 system access should resolve into the scheduler");
    assert!(!native_system.access().has_conservative_world_access());
    assert_eq!(
        native_system.thread_affinity(),
        crate::scene::ecs::SceneSystemThreadAffinity::WorkerSafe
    );
}

#[test]
fn native_host_api_v4_rejects_unknown_access_before_registry_registration() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .unwrap();
    let api = scope.api();
    let accesses = [ZrNativeSystemAccessV1 {
        abi_version: 1,
        size_bytes: core::mem::size_of::<ZrNativeSystemAccessV1>(),
        mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
        domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
        stable_id: ZrByteSlice::from_static(b"render.not_registered"),
    }];
    let system = ZrSystemRegistrationV2 {
        system_id: ZrByteSlice::from_static(b"weather.invalid_tick_v4"),
        stage: SystemStage::ORDER
            .iter()
            .position(|stage| *stage == SystemStage::Update)
            .unwrap() as u32,
        accesses: accesses.as_ptr(),
        access_count: accesses.len(),
        ..ZrSystemRegistrationV2::empty(4)
    };

    let status = unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) };

    assert_eq!(status.status_code(), ZrStatusCode::Error);
    drop(scope);
    assert_eq!(registry.plugin_systems().count(), 0);
}

#[test]
fn native_host_api_v4_rejects_empty_access_contract() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .unwrap();
    let api = scope.api();
    let system = ZrSystemRegistrationV2 {
        system_id: ZrByteSlice::from_static(b"weather.empty_access_tick_v4"),
        stage: SystemStage::ORDER
            .iter()
            .position(|stage| *stage == SystemStage::Update)
            .unwrap() as u32,
        ..ZrSystemRegistrationV2::empty(4)
    };

    let status = unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) };

    assert_eq!(status.status_code(), ZrStatusCode::Error);
    drop(scope);
    assert_eq!(registry.plugin_systems().count(), 0);
}

#[test]
fn native_host_api_v4_rejects_null_string_list_pointer_before_registry_registration() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .unwrap();
    let api = scope.api();
    let component = ZrComponentDescV1 {
        type_id: ZrByteSlice::from_static(b"weather.native_component"),
        display_name: ZrByteSlice::from_static(b"Native Weather Component"),
        schema: ZrByteSlice::from_static(br#"{"fields":[]}"#),
        storage_kind: 1,
        ..ZrComponentDescV1::empty(4)
    };
    let component_status =
        unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
    assert!(component_status.is_ok());
    let accesses = [ZrNativeSystemAccessV1 {
        abi_version: 1,
        size_bytes: core::mem::size_of::<ZrNativeSystemAccessV1>(),
        mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
        domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
        stable_id: ZrByteSlice::from_static(b"weather.native_component"),
    }];
    for field in ["set_names", "before", "after"] {
        let mut system = ZrSystemRegistrationV2 {
            system_id: ZrByteSlice::from_static(b"weather.null_string_list_pointer_v4"),
            stage: SystemStage::ORDER
                .iter()
                .position(|stage| *stage == SystemStage::Update)
                .unwrap() as u32,
            accesses: accesses.as_ptr(),
            access_count: accesses.len(),
            ..ZrSystemRegistrationV2::empty(4)
        };
        match field {
            "set_names" => system.set_count = 1,
            "before" => system.before_count = 1,
            "after" => system.after_count = 1,
            _ => unreachable!(),
        }

        let status = unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) };

        assert_eq!(status.status_code(), ZrStatusCode::Error, "field={field}");
    }
    drop(scope);
    assert_eq!(registry.plugin_systems().count(), 0);
}

#[test]
fn native_host_api_v4_rejects_known_foreign_access_without_host_capability() {
    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_component(ComponentTypeDescriptor::new(
            "render.visible",
            "render",
            "Visible",
        ))
        .unwrap();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .unwrap();
    let api = scope.api();
    let accesses = [ZrNativeSystemAccessV1 {
        abi_version: 1,
        size_bytes: core::mem::size_of::<ZrNativeSystemAccessV1>(),
        mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
        domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
        stable_id: ZrByteSlice::from_static(b"render.visible"),
    }];
    let system = ZrSystemRegistrationV2 {
        system_id: ZrByteSlice::from_static(b"weather.foreign_tick_v4"),
        stage: SystemStage::ORDER
            .iter()
            .position(|stage| *stage == SystemStage::Update)
            .unwrap() as u32,
        accesses: accesses.as_ptr(),
        access_count: accesses.len(),
        ..ZrSystemRegistrationV2::empty(4)
    };

    let status = unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) };

    assert_eq!(status.status_code(), ZrStatusCode::Error);
    drop(scope);
    assert_eq!(registry.plugin_systems().count(), 0);
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
    let scope = NativeHostBridgeCallScope::with_methods(
        table,
        [(
            slot,
            7,
            NativeBridgeMethodFn::from_rust(native_bridge_blocking_method),
        )],
    );
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
    let scope = NativeHostBridgeCallScope::with_methods(
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
    );

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
    let source = include_str!("../host_api_adapter.rs");

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
    let source = include_str!("../host_api_adapter.rs");
    let registry_source = include_str!("context_registry.rs");

    assert!(source.contains("DenseBridgeMethodTable"));
    assert!(source.contains("context.methods.get(interface_slot, method_slot)"));
    assert!(!source.contains("BTreeMap"));
    assert!(registry_source.contains("HOST_CONTEXT_PAGE_SLOTS"));
    assert!(!registry_source.contains("writer.slots.clone"));
    assert!(!registry_source.contains("Vec<Arc<HostContextSlot"));
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
