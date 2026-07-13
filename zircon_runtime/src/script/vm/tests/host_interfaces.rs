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
