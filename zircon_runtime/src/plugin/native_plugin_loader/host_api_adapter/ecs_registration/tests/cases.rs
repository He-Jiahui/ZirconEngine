use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrComponentDescV1, ZrNativeSystemAccessV1, ZrRuntimePluginHandle,
    ZrStatusCode, ZrSystemRegistrationV2, ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
    ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1, ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1,
};

use crate::core::framework::scene::{ComponentTypeDescriptor, SystemStage};
use crate::plugin::RuntimeExtensionRegistry;

use super::super::registration_policy::{
    NativeHostApiV4RegistrationPolicy, NativeHostApiV4RegistrationScope,
};

#[test]
fn native_host_api_v4_registers_systems_and_components_into_runtime_registry() {
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
    assert!(unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) }.is_ok());

    let accesses = [ZrNativeSystemAccessV1 {
        abi_version: 1,
        size_bytes: core::mem::size_of::<ZrNativeSystemAccessV1>(),
        mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
        domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
        stable_id: ZrByteSlice::from_static(b"weather.native_component"),
    }];
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
    assert!(unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) }.is_ok());
    drop(scope);

    let systems = registry.plugin_systems().collect::<Vec<_>>();
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].1.id, "weather.native_tick_v4");
    assert_eq!(registry.components().len(), 1);
    assert_eq!(registry.components()[0].plugin_id, "weather");
}

#[test]
fn native_host_api_v4_system_enters_schedule_with_declared_access() {
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
    unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
    let accesses = [ZrNativeSystemAccessV1 {
        abi_version: 1,
        size_bytes: core::mem::size_of::<ZrNativeSystemAccessV1>(),
        mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
        domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
        stable_id: ZrByteSlice::from_static(b"weather.native_component"),
    }];
    let system = ZrSystemRegistrationV2 {
        system_id: ZrByteSlice::from_static(b"weather.native_schedule_v4"),
        stage: SystemStage::ORDER
            .iter()
            .position(|stage| *stage == SystemStage::Update)
            .unwrap() as u32,
        accesses: accesses.as_ptr(),
        access_count: accesses.len(),
        thread_affinity: ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1,
        ..ZrSystemRegistrationV2::empty(4)
    };
    assert!(unsafe { (api.ecs.register_system.unwrap())(scope.handle(), &system) }.is_ok());
    drop(scope);

    let mut world = crate::scene::World::default();
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
fn native_host_api_v4_rejects_unknown_registration_handles() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .unwrap();
    let api = scope.api();
    let system = ZrSystemRegistrationV2 {
        system_id: ZrByteSlice::from_static(b"weather.native_tick_v4"),
        ..ZrSystemRegistrationV2::empty(4)
    };

    let status =
        unsafe { (api.ecs.register_system.unwrap())(ZrRuntimePluginHandle::new(9999), &system) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
}

#[test]
fn native_host_api_v4_exposes_bridge_domain_as_unsupported_until_connected() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .unwrap();
    let api = scope.api();

    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            1,
            2,
            core::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
}

#[test]
fn native_host_api_v4_preserves_dotted_plugin_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "net.rpc.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .expect("dotted plugin runtime module owner");
    let api = scope.api();
    let component = ZrComponentDescV1 {
        type_id: ZrByteSlice::from_static(b"net.rpc.NativePayload"),
        display_name: ZrByteSlice::from_static(b"Native RPC Payload"),
        ..ZrComponentDescV1::empty(4)
    };

    let status = unsafe { (api.ecs.register_component.unwrap())(scope.handle(), &component) };
    drop(scope);

    assert!(status.is_ok());
    assert_eq!(registry.components().len(), 1);
    assert_eq!(registry.components()[0].plugin_id, "net.rpc");
}
