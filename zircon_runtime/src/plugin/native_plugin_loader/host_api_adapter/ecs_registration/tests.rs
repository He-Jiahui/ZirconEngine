use std::marker::PhantomData;
use std::sync::Arc;

use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrComponentDescV1, ZrRuntimePluginHandle, ZrStatusCode,
    ZrSystemRegistrationV1,
};

use crate::core::framework::scene::SystemStage;
use crate::plugin::RuntimeExtensionRegistry;

use super::*;

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
fn native_host_api_v3_rejects_unknown_registration_handles() {
    let api = NativeHostApiV3RegistrationScope {
        handle: ZrRuntimePluginHandle::invalid(),
        lifetime: Arc::new(NativeHostRegistrationScopeState::default()),
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
