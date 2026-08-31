use zircon_runtime_interface::{
    ZrByteSlice, ZrComponentDescV1, ZrNativeSystemAccessV1, ZrStatusCode, ZrSystemRegistrationV2,
    ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1, ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1,
    ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1, ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1,
    ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1,
};

use crate::core::framework::scene::{ComponentTypeDescriptor, SystemStage};
use crate::plugin::RuntimeExtensionRegistry;

use super::*;

#[test]
fn native_host_api_v4_registration_policy_owner_is_folder_backed() {
    let root = include_str!("mod.rs");
    let context = include_str!("context.rs");
    let policy = include_str!("policy.rs");
    let scope = include_str!("scope.rs");

    for child in ["mod context;", "mod policy;", "mod scope;"] {
        assert!(
            root.contains(child),
            "registration-policy root should mount {child}"
        );
    }
    assert!(root.contains("pub use policy::NativeHostApiV4RegistrationPolicy;"));
    assert!(root.contains("pub use scope::NativeHostApiV4RegistrationScope;"));
    assert!(root.contains("pub(super) use context::NativeHostApiV4RegistrationContext;"));
    assert!(!root.contains("pub struct NativeHostApiV4RegistrationPolicy"));
    assert!(!root.contains("pub struct NativeHostApiV4RegistrationScope"));
    assert!(!root.contains("impl Drop for NativeHostApiV4RegistrationScope"));

    assert!(policy.contains("pub struct NativeHostApiV4RegistrationPolicy"));
    assert!(policy.contains("impl NativeHostApiV4RegistrationPolicy"));
    assert!(scope.contains("pub struct NativeHostApiV4RegistrationScope"));
    assert!(scope.contains("impl Drop for NativeHostApiV4RegistrationScope"));
    assert!(scope.contains("self.lifetime.close_and_wait();"));
    assert!(scope.contains("remove_context(self.handle.raw());"));
    assert!(context.contains("pub(in super::super) struct NativeHostApiV4RegistrationContext"));
    assert!(context.contains("pub(in super::super) fn v3_context(&self)"));
    assert!(policy.contains("pub(in super::super) granted_capabilities"));
    assert!(policy.contains("pub(in super::super) known_resource_ids"));

    for (path, source, budget) in [
        ("registration_policy/mod.rs", root, 20),
        ("registration_policy/context.rs", context, 80),
        ("registration_policy/policy.rs", policy, 80),
        ("registration_policy/scope.rs", scope, 160),
    ] {
        assert!(
            source.lines().count() < budget,
            "{path} should remain below its responsibility budget"
        );
    }
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
