use super::support::*;
use super::*;

#[test]
fn stale_generation_resolves_to_new_function() {
    let manager = VmPluginManager::mock();
    let mut package = test_package("callback-owner", "1.0.0");
    package.manifest.capabilities = CapabilitySet::default().with(VM_SYSTEM_CAPABILITY);
    let slot = manager.load_package(package.clone()).unwrap();
    let caller = VmInterfaceCaller::new(slot, 1, package.manifest.capabilities.clone());
    let mut callback = manager
        .host_interfaces()
        .register_system(
            &caller,
            "callback-owner.update",
            VmSystemStage::Update,
            "main",
            "tick",
        )
        .unwrap();

    assert_eq!(
        manager.invoke_callback(&mut callback, &[]).unwrap(),
        Some(ScriptHostValue::Null)
    );
    assert_eq!(callback.generation, 1);

    manager.hot_reload_slot(slot, package).unwrap();
    assert_eq!(manager.slot(slot).unwrap().generation, 2);
    assert_eq!(
        manager.invoke_callback(&mut callback, &[]).unwrap(),
        Some(ScriptHostValue::Null)
    );
    assert_eq!(callback.generation, 2);
}

#[test]
fn unauthorized_channel_returns_capability_denied() {
    let registry = VmHostInterfaceRegistry::default();
    let caller = VmInterfaceCaller::new(
        super::super::PluginSlotId::new(7),
        1,
        CapabilitySet::default(),
    );

    let errors = [
        registry
            .register_system(
                &caller,
                "game.update",
                VmSystemStage::Update,
                "main",
                "update",
            )
            .unwrap_err(),
        registry
            .register_behavior_node(&caller, "game.task", "Game Task", "main", "task")
            .unwrap_err(),
        registry
            .register_rpc_handler(
                &caller,
                "game.rpc",
                crate::core::framework::net::RpcPayloadSchema::for_type_path("game.rpc.v1"),
                "main",
                "rpc",
            )
            .unwrap_err(),
        registry
            .register_editor_operation(&caller, "Game.Asset.Open", "main", "open")
            .unwrap_err(),
    ];

    for error in errors {
        assert!(matches!(
            error,
            VmHostInterfaceError::CapabilityDenied { .. }
        ));
    }
}

#[test]
fn vm_bt_node_executes_in_tree() {
    let manager = VmPluginManager::mock();
    let mut package = test_package("bt-owner", "1.0.0");
    package.manifest.capabilities = CapabilitySet::default().with(VM_BT_NODE_CAPABILITY);
    let slot = manager.load_package(package.clone()).unwrap();
    let caller = VmInterfaceCaller::new(slot, 1, package.manifest.capabilities);
    manager
        .host_interfaces()
        .register_behavior_node(&caller, "script.task", "Script Task", "ai", "tick")
        .unwrap();

    let mut tree_leaf = manager.registered_behavior_nodes().remove(0).callback;
    let leaf_succeeded = manager.invoke_callback(&mut tree_leaf, &[]).is_ok();

    assert!(leaf_succeeded);
    assert_eq!(tree_leaf.slot, slot);
}

#[test]
fn authorized_rpc_and_editor_channels_publish_active_descriptors() {
    let manager = VmPluginManager::mock();
    let mut package = test_package("tool-owner", "1.0.0");
    package.manifest.capabilities = CapabilitySet::default()
        .with(VM_RPC_HANDLER_CAPABILITY)
        .with(VM_EDITOR_OPERATION_CAPABILITY);
    let slot = manager.load_package(package.clone()).unwrap();
    let caller = VmInterfaceCaller::new(slot, 1, package.manifest.capabilities);
    let registry = manager.host_interfaces();
    registry
        .register_rpc_handler(
            &caller,
            "game.rpc",
            crate::core::framework::net::RpcPayloadSchema::for_type_path("game.rpc.v1"),
            "main",
            "rpc",
        )
        .unwrap();
    registry
        .register_editor_operation(&caller, "Game.Asset.Open", "main", "open")
        .unwrap();

    assert_eq!(manager.registered_rpc_handlers()[0].id, "game.rpc");
    assert_eq!(
        manager.registered_rpc_handlers()[0]
            .payload_schema
            .schema_id(),
        "game.rpc.v1"
    );
    assert_eq!(
        manager.registered_editor_operations()[0].operation,
        "Game.Asset.Open"
    );
}

#[test]
fn stable_host_interface_queries_reuse_the_published_snapshot() {
    let manager = VmPluginManager::mock();
    let mut package = test_package("stable-interface-owner", "1.0.0");
    package.manifest.capabilities = CapabilitySet::default().with(VM_SYSTEM_CAPABILITY);
    let slot = manager.load_package(package.clone()).unwrap();
    let registry = manager.host_interfaces();
    registry
        .register_system(
            &VmInterfaceCaller::new(slot, 1, package.manifest.capabilities.clone()),
            "stable.update",
            VmSystemStage::Update,
            "main",
            "tick",
        )
        .unwrap();

    let published = registry.active_snapshot();
    for _ in 0..128 {
        assert_eq!(manager.registered_systems(VmSystemStage::Update).len(), 1);
    }
    let reused = registry.active_snapshot();

    assert!(Arc::ptr_eq(&published, &reused));
}

#[test]
fn stable_package_name_queries_reuse_the_published_active_index() {
    let manager = VmPluginManager::mock();
    let package = test_package("stable-package-owner", "1.0.0");
    let slot = manager.load_package(package).unwrap();

    let published = manager.active_plugin_snapshot();
    for _ in 0..128 {
        assert_eq!(
            manager
                .slot_for_package_name("stable-package-owner")
                .unwrap(),
            slot
        );
        assert_eq!(manager.active_generation(slot).unwrap(), 1);
    }
    let reused = manager.active_plugin_snapshot();

    assert!(Arc::ptr_eq(&published, &reused));
}

#[test]
fn staged_generation_registrations_publish_once_at_lifecycle_commit() {
    let manager = VmPluginManager::mock();
    let mut package = test_package("staged-interface-owner", "1.0.0");
    package.manifest.capabilities = CapabilitySet::default().with(VM_SYSTEM_CAPABILITY);
    let slot = manager.load_package(package.clone()).unwrap();
    let registry = manager.host_interfaces();
    let published = registry.active_snapshot();
    let staged = VmInterfaceCaller::new(slot, 2, package.manifest.capabilities);

    for index in 0..128 {
        registry
            .register_system(
                &staged,
                format!("staged.update.{index}"),
                VmSystemStage::Update,
                "main",
                &format!("tick_{index}"),
            )
            .unwrap();
    }

    assert!(Arc::ptr_eq(&published, &registry.active_snapshot()));
    assert!(manager.registered_systems(VmSystemStage::Update).is_empty());
}

#[test]
fn reload_and_unload_publish_the_active_interface_generation() {
    let manager = VmPluginManager::mock();
    let mut package = test_package("lifecycle-interface-owner", "1.0.0");
    package.manifest.capabilities = CapabilitySet::default().with(VM_SYSTEM_CAPABILITY);
    let slot = manager.load_package(package.clone()).unwrap();
    let registry = manager.host_interfaces();
    registry
        .register_system(
            &VmInterfaceCaller::new(slot, 1, package.manifest.capabilities.clone()),
            "lifecycle.update",
            VmSystemStage::Update,
            "main",
            "tick_v1",
        )
        .unwrap();
    let first = registry.active_snapshot();

    manager.hot_reload_slot(slot, package.clone()).unwrap();
    let reloaded = registry.active_snapshot();
    assert!(!Arc::ptr_eq(&first, &reloaded));
    registry
        .register_system(
            &VmInterfaceCaller::new(slot, 2, package.manifest.capabilities),
            "lifecycle.update",
            VmSystemStage::Update,
            "main",
            "tick_v2",
        )
        .unwrap();
    assert_eq!(
        manager.registered_systems(VmSystemStage::Update)[0]
            .callback
            .generation,
        2
    );

    manager.unload_slot(slot).unwrap();
    let unloaded = registry.active_snapshot();
    assert!(!Arc::ptr_eq(&reloaded, &unloaded));
    assert!(manager.registered_systems(VmSystemStage::Update).is_empty());
}
