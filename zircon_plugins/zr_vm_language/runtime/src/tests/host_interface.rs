use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::script::{
    CapabilitySet, VmHostInterfaceError, VmInterfaceCaller, VmPluginManagementPolicy,
    VmPluginManager, VmPluginManifest, VmPluginPackage, VmSystemStage, VM_BT_NODE_CAPABILITY,
    VM_EDITOR_OPERATION_CAPABILITY, VM_RPC_HANDLER_CAPABILITY, VM_SYSTEM_CAPABILITY,
};

#[test]
fn stale_generation_resolves_to_new_function() {
    let manager = VmPluginManager::mock();
    let package = package("callback-owner", [VM_SYSTEM_CAPABILITY]);
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

    manager.hot_reload_slot(slot, package).unwrap();
    assert_eq!(
        manager.invoke_callback(&mut callback, &[]).unwrap(),
        Some(ScriptHostValue::Null)
    );
    assert_eq!(callback.generation, 2);
}

#[test]
fn vm_bt_node_executes_in_tree() {
    let manager = VmPluginManager::mock();
    let package = package("bt-owner", [VM_BT_NODE_CAPABILITY]);
    let slot = manager.load_package(package.clone()).unwrap();
    let caller = VmInterfaceCaller::new(slot, 1, package.manifest.capabilities);
    manager
        .host_interfaces()
        .register_behavior_node(&caller, "script.task", "Script Task", "ai", "tick")
        .unwrap();

    let mut nodes = manager.registered_behavior_nodes();
    let mut callback = nodes.remove(0).callback;
    assert!(manager.invoke_callback(&mut callback, &[]).is_ok());
}

#[test]
fn unauthorized_channel_returns_capability_denied() {
    let manager = VmPluginManager::mock();
    let package = package("denied-owner", []);
    let slot = manager.load_package(package).unwrap();
    let caller = VmInterfaceCaller::new(slot, 1, CapabilitySet::default());
    let registry = manager.host_interfaces();

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
                zircon_runtime::core::framework::net::RpcPayloadSchema::for_type_path(
                    "game.rpc.v1",
                ),
                "main",
                "rpc",
            )
            .unwrap_err(),
        registry
            .register_editor_operation(&caller, "Game.Asset.Open", "main", "open")
            .unwrap_err(),
    ];
    assert!(errors
        .iter()
        .all(|error| matches!(error, VmHostInterfaceError::CapabilityDenied { .. })));
}

#[test]
fn authorized_rpc_and_editor_channels_are_visible() {
    let manager = VmPluginManager::mock();
    let package = package(
        "tool-owner",
        [VM_RPC_HANDLER_CAPABILITY, VM_EDITOR_OPERATION_CAPABILITY],
    );
    let slot = manager.load_package(package.clone()).unwrap();
    let caller = VmInterfaceCaller::new(slot, 1, package.manifest.capabilities);
    let registry = manager.host_interfaces();
    registry
        .register_rpc_handler(
            &caller,
            "game.rpc",
            zircon_runtime::core::framework::net::RpcPayloadSchema::for_type_path("game.rpc.v1"),
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

fn package<const N: usize>(name: &str, capabilities: [&str; N]) -> VmPluginPackage {
    let capabilities = capabilities
        .into_iter()
        .fold(CapabilitySet::default(), |set, capability| {
            set.with(capability)
        });
    VmPluginPackage {
        manifest: VmPluginManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            entry: "main".to_string(),
            capabilities,
            management: VmPluginManagementPolicy::default(),
        },
        zr_vm_project: None,
        bytecode: vec![1],
    }
}
